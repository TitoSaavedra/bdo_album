use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::db::repositories::log_repo::LogRepository;
use crate::db::repositories::pab_repo::PabRepository;
use crate::db::repositories::pending_pab_repo::{PendingPabRow, PendingPabUploadRepository};
use crate::db::repositories::preset_repo::PresetRepository;
use crate::events::Sink;

use super::pg_notify_loop;
use super::r2::R2Client;
use super::service;

/// Safety net only — real wake-ups come from `NOTIFY pab_upload_queued` (fired
/// by the album when a .pab is dropped into its "Presets to download" modal,
/// see `pg_notify_loop`). This just catches anything missed while the LISTEN
/// connection was down.
const FALLBACK_POLL_INTERVAL: Duration = Duration::from_secs(300);

/// Background worker: drains the album's staged-PAB queue as soon as it's
/// queued (via `LISTEN pab_upload_queued`) and uploads to R2 through
/// `service::upload_pab` — album never talks to R2 directly for this feature,
/// it only stages the raw filename and bytes (see the migration's comment on
/// `album_pending_pab_uploads`). All validation (preset ID, preset exists, no
/// PAB yet) happens here. Runs alongside `preset_modifications`'s loop, in both
/// the desktop GUI and the headless `download_daemon` binary.
pub async fn run_loop(sink: Arc<dyn Sink>, pool: PgPool) {
    // Drain anything already queued before we start listening (e.g. queued
    // while this process was down).
    drain_queue(&sink, &pool).await;

    pg_notify_loop::run(
        &sink,
        &pool,
        "pab_upload_queued",
        "pab_uploads",
        FALLBACK_POLL_INTERVAL,
        || drain_queue(&sink, &pool),
    )
    .await;
}

/// Extracts the preset ID from a PAB filename containing `ID<digits>`
/// (e.g. `Guardian_ID1123335.pab`). Shared by the album-import tauri command
/// and this worker so both read filenames the same way.
pub fn extract_preset_id(filename: &str) -> Option<i64> {
    let pos = filename.find("ID")?;
    let after = &filename[pos + 2..];
    let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
    if end == 0 { return None; }
    after[..end].parse().ok()
}

async fn drain_queue(sink: &Arc<dyn Sink>, pool: &PgPool) {
    loop {
        match PendingPabUploadRepository::next_pending(pool).await {
            Ok(Some(pending)) => process_one(sink, pool, pending).await,
            Ok(None) => break,
            Err(e) => {
                LogRepository::insert(sink.as_ref(), pool, None, "ERR", "pab_uploads",
                    &format!("queue check failed: {e}")).await.ok();
                break;
            }
        }
    }
}

async fn process_one(sink: &Arc<dyn Sink>, pool: &PgPool, pending: PendingPabRow) {
    let filename = &pending.filename;

    let Some(preset_id) = extract_preset_id(filename) else {
        let msg = format!("{filename}: no preset ID in filename");
        PendingPabUploadRepository::mark_failed(pool, pending.id, "no preset ID in filename").await.ok();
        LogRepository::insert(sink.as_ref(), pool, None, "WARN", "pab_uploads", &msg).await.ok();
        return;
    };

    match PresetRepository::get_with_class(pool, preset_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            let msg = format!("{filename}: preset {preset_id} not found");
            PendingPabUploadRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "WARN", "pab_uploads", &msg).await.ok();
            return;
        }
        Err(e) => {
            let msg = format!("{filename}: preset {preset_id} lookup failed: {e}");
            PendingPabUploadRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "pab_uploads", &msg).await.ok();
            return;
        }
    }

    // A preset that already has a PAB keeps it — the staged file is a duplicate,
    // so just purge it.
    match PabRepository::find_by_preset(pool, preset_id).await {
        Ok(Some(_)) => {
            PendingPabUploadRepository::delete(pool, pending.id).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "INFO", "pab_uploads",
                &format!("{filename}: preset {preset_id} already has a PAB, skipped")).await.ok();
            return;
        }
        Ok(None) => {}
        Err(e) => {
            let msg = format!("{filename}: PAB lookup failed: {e}");
            PendingPabUploadRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "pab_uploads", &msg).await.ok();
            return;
        }
    }

    let r2 = match R2Client::from_env() {
        Ok(r2) => r2,
        Err(e) => {
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "pab_uploads",
                &format!("{filename}: R2 not configured, skipping: {e}")).await.ok();
            return;
        }
    };

    match service::upload_pab(pool, &r2, preset_id, filename, pending.pab_data).await {
        Ok(db_path) => {
            // Purge the staging row — the scraper_preset_pabs row written by
            // upload_pab is now the copy of record.
            PendingPabUploadRepository::delete(pool, pending.id).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "SYNC", "pab_uploads",
                &format!("{filename}: uploaded → {db_path}")).await.ok();
        }
        Err(e) => {
            let msg = format!("{filename}: upload failed: {e}");
            PendingPabUploadRepository::mark_failed(pool, pending.id, &msg).await.ok();
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", "pab_uploads", &msg).await.ok();
        }
    }
}
