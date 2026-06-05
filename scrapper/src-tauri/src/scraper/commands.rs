use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, State};

use crate::core::errors::{AppError, Result};
use crate::core::state::AppState;
use crate::db::repositories::{log_repo::LogRepository, session_repo::SessionRepository};
use crate::events::Events;

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
    days:        Vec<String>,
    regions:     Vec<String>,
    classes:     Vec<serde_json::Value>,
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

    Events::scrapper_started(&app);

    let days    = if days.is_empty()    { vec!["20","30","60","90","180","365","ever"].into_iter().map(String::from).collect() } else { days };
    let regions = if regions.is_empty() { vec!["eu","na","ru","jp","kr","tw","sa","sea","asia","mena"].into_iter().map(String::from).collect() } else { regions };
    let classes = if classes.is_empty() { vec![serde_json::json!("all")] } else { classes };

    let classes_str: Vec<String> = classes.iter().map(|v| {
        if let Some(s) = v.as_str() { s.to_string() }
        else if let Some(n) = v.as_i64() { n.to_string() }
        else { "?".to_string() }
    }).collect();

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "session",
        &format!("Session #{} started — parallelism={} | classes=[{}] | days=[{}] | regions=[{}]",
            session_id, parallelism,
            classes_str.join(", "),
            days.join(", "),
            regions.join(", "),
        ),
    ).await.ok();

    tauri::async_runtime::spawn(super::service::run_session(
        app, pool, cancel, session_id, parallelism, days, regions, classes,
    ));

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
        "skipped":        r.skipped,
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
pub async fn get_preset_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value> {
    use crate::db::repositories::preset_repo::PresetRepository;
    let stats = PresetRepository::get_stats(&state.pool).await?;
    Ok(stats)
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
