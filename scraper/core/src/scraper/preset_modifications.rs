use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::db::repositories::log_repo::LogRepository;
use crate::db::repositories::modification_repo::{ModificationRepository, PendingModificationRepository, PendingRow};
use crate::errors::Result;
use crate::events::Sink;

use super::r2::R2Client;

const POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Background worker: drains the album's staged-modification-image queue
/// every 60s and uploads to R2 — album never talks to R2 directly for this
/// feature, it only stages raw pasted bytes (see the migration's comment on
/// `album_pending_preset_modifications`). Runs alongside `auto_download`'s
/// loop, in both the desktop GUI and the headless `download_daemon` binary.
pub async fn run_loop(sink: Arc<dyn Sink>, pool: PgPool) {
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        loop {
            match PendingModificationRepository::next_pending(&pool).await {
                Ok(Some(pending)) => process_one(&sink, &pool, pending).await,
                Ok(None) => break,
                Err(e) => {
                    LogRepository::insert(sink.as_ref(), &pool, None, "ERR", "preset_modifications",
                        &format!("queue check failed: {e}")).await.ok();
                    break;
                }
            }
        }
    }
}

async fn process_one(sink: &Arc<dyn Sink>, pool: &PgPool, pending: PendingRow) {
    let r2 = match R2Client::from_env() {
        Ok(r2) => r2,
        Err(e) => {
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications",
                &format!("R2 not configured, skipping: {e}")).await.ok();
            return;
        }
    };

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let key_1 = match upload_one(&r2, pending.preset_id, ts, 1, &pending.image_1_data).await {
        Ok(k) => k,
        Err(e) => {
            let msg = format!("preset {}: image 1 upload failed: {e}", pending.preset_id);
            PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
            return;
        }
    };

    let key_2 = match &pending.image_2_data {
        Some(bytes) => match upload_one(&r2, pending.preset_id, ts, 2, bytes).await {
            Ok(k) => Some(k),
            Err(e) => {
                let msg = format!("preset {}: image 2 upload failed: {e}", pending.preset_id);
                PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
                LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
                return;
            }
        },
        None => None,
    };

    match ModificationRepository::insert(pool, pending.preset_id, &key_1, key_2.as_deref()).await {
        Ok(()) => {
            // Purge the staging row — the finished row above is now the copy of record.
            PendingModificationRepository::delete(pool, pending.id).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "SYNC", "preset_modifications",
                &format!("preset {}: modification uploaded → {key_1}", pending.preset_id)).await.ok();
        }
        Err(e) => {
            let msg = format!("preset {}: insert failed: {e}", pending.preset_id);
            PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
        }
    }
}

/// Decodes, resizes to a large-but-bounded long edge (viewed full-size in
/// album's PresetDetail hero, so bigger than face_grid's 512/256 working
/// sizes), re-encodes as WebP, uploads to R2, and returns the key (not the
/// full URL `upload()` returns — every existing caller discards that and
/// keeps the key it built itself, same convention here).
async fn upload_one(r2: &R2Client, preset_id: i64, ts: u128, slot: u8, bytes: &[u8]) -> Result<String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| crate::errors::AppError::Scrape(format!("invalid image: {e}")))?;
    let resized = img.thumbnail(1600, 1600);

    let mut buf: Vec<u8> = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::WebP)
        .map_err(|e| crate::errors::AppError::Scrape(format!("image encode: {e}")))?;

    let key = format!("preset-modifications/{preset_id}/{ts}-{slot}.webp");
    let _ = r2.upload(&key, buf).await?;
    Ok(key)
}
