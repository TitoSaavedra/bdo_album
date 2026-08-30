use std::time::Duration;

use sqlx::PgPool;
use tauri::AppHandle;

use crate::db::repositories::auto_download_repo::AutoDownloadRepository;
use crate::db::repositories::log_repo::LogRepository;
use crate::events::{AutoDownloadStatus, Events};

use super::browser::{BrowserSession, PabDownloadOutcome};
use super::r2::R2Client;
use super::service;

const POLL_INTERVAL: Duration = Duration::from_secs(60);
const BETWEEN_ITEMS: Duration = Duration::from_secs(5);

/// Background worker: while the app is open, periodically checks for presets the
/// user queued from the album ("Enviar a descarga automática") and downloads their
/// `.pab` via the authenticated Playwright session (see `session.rs` for how that
/// session gets imported). Independent of the "Iniciar Scraping" button/session.
pub async fn run_loop(app: AppHandle, pool: PgPool) {
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        match AutoDownloadRepository::next_pending(&pool).await {
            Ok(Some(_)) => process_queue(&app, &pool).await,
            Ok(None) => {}
            Err(e) => {
                LogRepository::insert(&app, &pool, None, "ERR", "auto_download",
                    &format!("queue check failed: {e}")).await.ok();
            }
        }
    }
}

/// Drains every currently-queued preset in one browser session (avoids relaunching
/// Chromium per item) and stops early if Garmoth's monthly download quota is hit.
async fn process_queue(app: &AppHandle, pool: &PgPool) {
    let r2 = match R2Client::from_env() {
        Ok(r2) => r2,
        Err(e) => {
            LogRepository::insert(app, pool, None, "ERR", "auto_download",
                &format!("R2 not configured, skipping auto-download: {e}")).await.ok();
            return;
        }
    };

    let browser = match BrowserSession::new(app, pool, None).await {
        Ok(b) => b,
        Err(e) => {
            LogRepository::insert(app, pool, None, "ERR", "auto_download",
                &format!("failed to start browser: {e}")).await.ok();
            return;
        }
    };

    let downloads_dir = dirs::download_dir()
        .unwrap_or_else(std::env::temp_dir);

    loop {
        let preset_id = match AutoDownloadRepository::next_pending(pool).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                Events::auto_download_status(app, AutoDownloadStatus::Idle);
                break;
            }
            Err(e) => {
                LogRepository::insert(app, pool, None, "ERR", "auto_download",
                    &format!("queue check failed: {e}")).await.ok();
                Events::auto_download_status(app, AutoDownloadStatus::Idle);
                break;
            }
        };

        Events::auto_download_status(app, AutoDownloadStatus::Downloading { preset_id });

        match browser.download_pab(preset_id, &downloads_dir).await {
            Ok(PabDownloadOutcome::Saved { path, filename }) => {
                let bytes = match tokio::fs::read(&path).await {
                    Ok(b) => b,
                    Err(e) => {
                        let msg = format!("preset {preset_id}: read downloaded file {}: {e}", path.display());
                        AutoDownloadRepository::mark_failed(pool, preset_id, &msg).await.ok();
                        LogRepository::insert(app, pool, None, "ERR", "auto_download", &msg).await.ok();
                        continue;
                    }
                };

                match service::upload_pab(pool, &r2, preset_id, &filename, bytes).await {
                    Ok(db_path) => {
                        AutoDownloadRepository::mark_done(pool, preset_id).await.ok();
                        LogRepository::insert(app, pool, None, "SYNC", "auto_download",
                            &format!("preset {preset_id}: downloaded to {} and uploaded → {db_path}", path.display())).await.ok();
                    }
                    Err(e) => {
                        let msg = format!("preset {preset_id}: upload failed: {e}");
                        AutoDownloadRepository::mark_failed(pool, preset_id, &msg).await.ok();
                        LogRepository::insert(app, pool, None, "ERR", "auto_download", &msg).await.ok();
                    }
                }
            }
            Ok(PabDownloadOutcome::QuotaExceeded { used, limit }) => {
                LogRepository::insert(app, pool, None, "INFO", "auto_download",
                    &format!("Garmoth monthly download quota reached ({used}/{limit}) — will retry later")).await.ok();
                Events::auto_download_status(app, AutoDownloadStatus::QuotaExceeded { used, limit });
                break;
            }
            Err(e) => {
                let msg = format!("preset {preset_id}: {e}");
                AutoDownloadRepository::mark_failed(pool, preset_id, &msg).await.ok();
                LogRepository::insert(app, pool, None, "ERR", "auto_download", &msg).await.ok();
            }
        }

        tokio::time::sleep(BETWEEN_ITEMS).await;
    }
}
