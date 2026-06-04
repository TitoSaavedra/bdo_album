use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, State};

use crate::core::errors::{AppError, Result};
use crate::core::state::AppState;
use crate::db::repositories::{log_repo::LogRepository, session_repo::SessionRepository};
use crate::events::{Events, ScrapperDone, ScrapperError, ScrapperPhase};

use super::browser::BrowserSession;
use super::r2::R2Client;

/// Returns true if AppState is managed (DB connected), false otherwise.
#[tauri::command]
pub async fn get_db_status(app: AppHandle) -> bool {
    app.try_state::<AppState>().is_some()
}

#[tauri::command]
pub async fn get_classes(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>> {
    use crate::db::repositories::class_repo::ClassRepository;
    let rows = ClassRepository::get_all(&state.pool).await?;
    let result = rows.into_iter().map(|r| serde_json::json!({
        "id":       r.id,
        "name":     r.display,
        "icon_svg": r.icon_svg,
    })).collect();
    Ok(result)
}

#[tauri::command]
pub async fn run_scraper(
    app:         AppHandle,
    state:       State<'_, AppState>,
    parallelism: usize,
) -> Result<i64> {
    {
        let guard = state.current_session.lock().unwrap();
        if guard.is_some() {
            return Err(AppError::Scrape("scraper already running".into()));
        }
    }

    let parallelism = parallelism.max(1);
    let pool        = state.pool.clone();
    let cancel      = state.cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    let session_id = SessionRepository::create(&pool, true).await?;
    *state.current_session.lock().unwrap() = Some(session_id);

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "session",
        &format!("Session #{} started (parallelism={})", session_id, parallelism)).await.ok();

    Events::scrapper_started(&app);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let started = std::time::Instant::now();

        // ── Init R2 client ───────────────────────────────────
        let r2 = match R2Client::from_env() {
            Ok(r2) => r2,
            Err(e) => {
                finish_with_error(&app_clone, &pool, session_id, e).await;
                return;
            }
        };

        // ── Init browser ─────────────────────────────────────
        Events::sync_loading(&app_clone, "Starting browser...");
        let browser = match BrowserSession::new().await {
            Ok(b) => b,
            Err(e) => {
                finish_with_error(&app_clone, &pool, session_id, e).await;
                return;
            }
        };

        // Wait up to 30s for CF to set the clearance cookie after warm-up
        Events::sync_loading(&app_clone, "Waiting for CF clearance...");
        let cf_token = browser.wait_for_cf_clearance(30).await.unwrap_or_default();
        let cf_ok = !cf_token.is_empty();
        eprintln!("[scraper] cf_clearance obtained: {}", cf_ok);
        LogRepository::insert(&app_clone, &pool, Some(session_id), "INFO", "browser",
            if cf_ok { "CF clearance obtained" } else { "CF clearance not found — proceeding without it" }
        ).await.ok();

        // ── Phase 1: Fetch JSON ───────────────────────────────
        Events::sync_loading(&app_clone, "Fetching presets...");
        LogRepository::insert(&app_clone, &pool, Some(session_id), "ORCH", "fetch", "Fetch phase started").await.ok();

        let fetch_result = super::service::run_fetch(
            &app_clone, &pool, cf_token, session_id, cancel.clone(), parallelism,
        ).await;

        let fetch = match fetch_result {
            Ok(r) => r,
            Err(e) => {
                finish_with_error(&app_clone, &pool, session_id, e).await;
                return;
            }
        };

        LogRepository::insert(&app_clone, &pool, Some(session_id), "ORCH", "fetch",
            &format!("Fetch done — {} new presets, {} errors", fetch.total_fetched, fetch.total_errors)
        ).await.ok();

        if cancel.load(Ordering::Relaxed) {
            LogRepository::insert(&app_clone, &pool, Some(session_id), "WARN", "session", "Cancelled after fetch").await.ok();
            end_cancelled(&app_clone, &pool, session_id, fetch.total_fetched, 0, 0, fetch.total_errors, started).await;
            return;
        }

        // ── Phase 2: Download + Upload ────────────────────────
        Events::sync_loading(&app_clone, "Downloading & uploading images...");
        LogRepository::insert(&app_clone, &pool, Some(session_id), "ORCH", "images",
            &format!("Image phase started — {} presets queued", fetch.image_tasks.len())
        ).await.ok();

        let dl_result = super::service::run_download_upload(
            &app_clone, &browser, &r2, &pool,
            session_id, cancel.clone(), fetch.image_tasks,
        ).await;

        let dl = match dl_result {
            Ok(r) => r,
            Err(e) => {
                finish_with_error(&app_clone, &pool, session_id, e).await;
                return;
            }
        };

        LogRepository::insert(&app_clone, &pool, Some(session_id), "ORCH", "images",
            &format!("Images done — {} downloaded, {} uploaded, {} errors", dl.total_images, dl.total_uploaded, dl.total_errors)
        ).await.ok();

        let elapsed       = started.elapsed().as_secs();
        let was_cancelled = cancel.load(Ordering::Relaxed);
        let total_errors  = fetch.total_errors + dl.total_errors;

        {
            let state: tauri::State<'_, AppState> = app_clone.state();
            *state.current_session.lock().unwrap() = None;
        }

        let status = if was_cancelled { "cancelled" } else { "done" };
        SessionRepository::finish(
            &pool, session_id, status,
            fetch.total_fetched as i32,
            dl.total_images as i32,
            dl.total_uploaded as i32,
            total_errors as i32,
            elapsed as i32,
        ).await.ok();

        if was_cancelled {
            LogRepository::insert(&app_clone, &pool, Some(session_id), "WARN", "session", "Session cancelled").await.ok();
            Events::scrapper_cancelled(&app_clone);
        } else {
            LogRepository::insert(&app_clone, &pool, Some(session_id), "ORCH", "session",
                &format!("Session #{} done in {}s — {} presets, {} images, {} errors",
                    session_id, elapsed, fetch.total_fetched, dl.total_uploaded, total_errors)
            ).await.ok();
            Events::scrapper_done(&app_clone, ScrapperDone {
                total_fetched:  fetch.total_fetched,
                total_images:   dl.total_images,
                total_uploaded: dl.total_uploaded,
                errors:         total_errors,
                elapsed_secs:   elapsed,
            });
        }
    });

    Ok(session_id)
}

#[tauri::command]
pub async fn cancel_scraper(state: State<'_, AppState>) -> Result<()> {
    state.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    let rows = SessionRepository::get_recent(&state.pool, limit.unwrap_or(50)).await?;
    let result = rows.into_iter().map(|r| serde_json::json!({
        "id":             r.id,
        "started_at":     r.started_at.timestamp_millis(),
        "finished_at":    r.finished_at.map(|t| t.timestamp_millis()),
        "status":         r.status,
        "total_fetched":  r.total_fetched,
        "total_images":   r.total_images,
        "total_uploaded": r.total_uploaded,
        "errors":         r.errors,
        "elapsed_secs":   r.elapsed_secs,
        "cf_used":        r.cf_used,
    })).collect();
    Ok(result)
}

#[tauri::command]
pub async fn get_class_stats_cmd(
    state:      State<'_, AppState>,
    session_id: i64,
) -> Result<Vec<serde_json::Value>> {
    let rows = SessionRepository::get_class_stats(&state.pool, session_id).await?;
    let result = rows.into_iter().map(|r| serde_json::json!({
        "class_id":  r.class_id,
        "fetched":   r.fetched,
        "images_ok": r.images_ok,
        "errors":    r.errors,
    })).collect();
    Ok(result)
}

#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>> {
    let rows = LogRepository::get_recent(&state.pool, limit.unwrap_or(100)).await?;
    let result = rows.into_iter().map(|r| serde_json::json!({
        "id":         r.id,
        "ts":         r.ts.timestamp_millis(),
        "session_id": r.session_id,
        "tag":        r.tag,
        "source":     r.source,
        "msg":        r.msg,
    })).collect();
    Ok(result)
}

// ── Helpers ───────────────────────────────────────────────────

async fn finish_with_error(app: &AppHandle, pool: &sqlx::PgPool, session_id: i64, e: AppError) {
    LogRepository::insert(app, pool, Some(session_id), "ERR", "session",
        &format!("Session failed: {}", e)).await.ok();
    {
        let state: tauri::State<'_, AppState> = app.state();
        *state.current_session.lock().unwrap() = None;
    }
    SessionRepository::finish(pool, session_id, "error", 0, 0, 0, 1, 0).await.ok();
    Events::scrapper_error(app, ScrapperError {
        message: e.to_string(),
        phase:   ScrapperPhase::Fetch,
    });
}

async fn end_cancelled(
    app: &AppHandle, pool: &sqlx::PgPool, session_id: i64,
    fetched: usize, images: usize, uploaded: usize, errors: usize,
    started: std::time::Instant,
) {
    {
        let state: tauri::State<'_, AppState> = app.state();
        *state.current_session.lock().unwrap() = None;
    }
    SessionRepository::finish(
        pool, session_id, "cancelled",
        fetched as i32, images as i32, uploaded as i32, errors as i32,
        started.elapsed().as_secs() as i32,
    ).await.ok();
    Events::scrapper_cancelled(app);
}
