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

    // Names files `modificacion_{n}` (1-indexed, per preset) instead of a
    // timestamp — so `preset X`'s 3rd modification is recognizable by name
    // alone as `modificacion_3`, not an opaque millisecond count.
    let index = match ModificationRepository::count_by_preset(pool, pending.preset_id).await {
        Ok(n) => n + 1,
        Err(e) => {
            let msg = format!("preset {}: count failed: {e}", pending.preset_id);
            PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
            return;
        }
    };

    let key_1 = match upload_one(&r2, pending.preset_id, index, 1, &pending.image_1_data).await {
        Ok(k) => k,
        Err(e) => {
            let msg = format!("preset {}: image 1 upload failed: {e}", pending.preset_id);
            PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
            return;
        }
    };

    let key_2 = match &pending.image_2_data {
        Some(bytes) => match upload_one(&r2, pending.preset_id, index, 2, bytes).await {
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

    let pab_key = match &pending.pab_data {
        Some(bytes) => match upload_pab(&r2, pending.preset_id, index, bytes).await {
            Ok(k) => Some(k),
            Err(e) => {
                let msg = format!("preset {}: pab upload failed: {e}", pending.preset_id);
                PendingModificationRepository::mark_failed(pool, pending.id, &msg).await.ok();
                LogRepository::insert(sink.as_ref(), pool, None, "ERR", "preset_modifications", &msg).await.ok();
                return;
            }
        },
        None => None,
    };

    match ModificationRepository::insert(pool, pending.preset_id, &key_1, key_2.as_deref(), pab_key.as_deref()).await {
        Ok(()) => {
            // Purge the staging row — the finished row above is now the copy of record.
            PendingModificationRepository::delete(pool, pending.id).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "SYNC", "preset_modifications",
                &format!("preset {}: modification uploaded → {key_1}", pending.preset_id)).await.ok();

            // Tell any open Album instance to refresh this preset — same event
            // scraper's image/PAB pipeline already fires, so the existing
            // listener (which always refetches the full row) just works; null
            // image URLs here since scraper_presets' own images didn't change.
            sqlx::query("SELECT pg_notify('preset_uploaded', $1)")
                .bind(serde_json::json!({
                    "preset_id": pending.preset_id,
                    "class_id": pending.class_id,
                    "image_1_url": Option::<String>::None,
                    "image_2_url": Option::<String>::None,
                }).to_string())
                .execute(pool)
                .await
                .ok();
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
/// sizes), re-encodes as WebP, uploads to R2 under the raw key, and returns
/// the **DB-storage form** (leading `/`) — matching `service::upload_pab`'s
/// established split between the R2 key (no leading slash) and the value
/// stored in the DB (with one, so `r2_public_url || column` concatenates
/// into a valid URL — `scraper_presets.image_1_url` follows this exact
/// convention, e.g. `/images/Guardian/1123335/51372_1.png`).
async fn upload_one(r2: &R2Client, preset_id: i64, index: i64, slot: u8, bytes: &[u8]) -> Result<String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| crate::errors::AppError::Scrape(format!("invalid image: {e}")))?;
    let resized = img.thumbnail(1600, 1600);

    let mut buf: Vec<u8> = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::WebP)
        .map_err(|e| crate::errors::AppError::Scrape(format!("image encode: {e}")))?;

    let key = format!("preset-modifications/{preset_id}/modificacion_{index}-{slot}.webp");
    let _ = r2.upload(&key, buf).await?;
    Ok(format!("/{key}"))
}

/// Uploads the .pab as-is — a binary game format, never decoded/re-encoded
/// as an image. No file extension, matching how PAB files elsewhere in this
/// system are already keyed (see `service::upload_pab`). Same R2-key-vs-
/// DB-storage-form split as `upload_one` above.
async fn upload_pab(r2: &R2Client, preset_id: i64, index: i64, bytes: &[u8]) -> Result<String> {
    let key = format!("preset-modifications/{preset_id}/modificacion_{index}-pab");
    let _ = r2.upload(&key, bytes.to_vec()).await?;
    Ok(format!("/{key}"))
}
