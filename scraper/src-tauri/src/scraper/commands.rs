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
    mode:        Option<String>,
) -> Result<i64> {
    {
        let guard = state.current_session.lock().unwrap();
        if guard.is_some() {
            return Err(AppError::Scrape("scraper already running".into()));
        }
    }

    let parallelism = parallelism.max(1);
    let mode        = mode.unwrap_or_else(|| "both".to_string());
    let pool        = state.pool.clone();
    let cancel      = state.cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    SessionRepository::cancel_stale(&pool).await.ok();
    let session_id = SessionRepository::create(&pool, true).await?;
    *state.current_session.lock().unwrap() = Some(session_id);

    Events::scraper_started(&app);

    let days    = if days.is_empty()    { vec!["20","30","60","90","180","365","ever"].into_iter().map(String::from).collect() } else { days };
    let regions = if regions.is_empty() { vec!["eu","na","ru","jp","kr","tw","sa","sea","asia","mena"].into_iter().map(String::from).collect() } else { regions };
    let classes = if classes.is_empty() { vec![serde_json::json!("all")] } else { classes };

    let classes_str: Vec<String> = classes.iter().map(|v| {
        if let Some(s) = v.as_str() { s.to_string() }
        else if let Some(n) = v.as_i64() { n.to_string() }
        else { "?".to_string() }
    }).collect();

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "session",
        &format!("Session #{} started — mode={} | parallelism={} | classes=[{}] | days=[{}] | regions=[{}]",
            session_id, mode, parallelism,
            classes_str.join(", "),
            days.join(", "),
            regions.join(", "),
        ),
    ).await.ok();

    tauri::async_runtime::spawn(super::service::run_session(
        app, pool, cancel, session_id, parallelism, days, regions, classes, mode,
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
        "total_updated":  r.total_updated,
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
pub async fn get_session_totals(
    state: State<'_, AppState>,
) -> Result<serde_json::Value> {
    let t = SessionRepository::get_totals(&state.pool).await?;
    Ok(serde_json::json!({
        "count":          t.count,
        "total_fetched":  t.total_fetched,
        "total_updated":  t.total_updated,
        "total_images":   t.total_images,
        "total_uploaded": t.total_uploaded,
        "errors":         t.errors,
        "skipped":        t.skipped,
    }))
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

#[derive(serde::Serialize)]
pub struct ImportPabResult {
    pub found:           usize,
    pub patched:         usize,
    pub not_found_count: usize,
    pub not_found_names: Vec<String>,
    pub zip_base64:      Option<String>,
}

fn extract_preset_id(filename: &str) -> Option<i64> {
    let pos = filename.find("ID")?;
    let after = &filename[pos + 2..];
    let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
    if end == 0 { return None; }
    after[..end].parse().ok()
}

#[tauri::command]
pub async fn import_pab_files(
    app:   AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportPabResult> {
    use base64::Engine as _;
    use std::io::Write as _;
    use crate::db::repositories::{log_repo::LogRepository, preset_repo::PresetRepository, pab_repo::PabRepository};
    use crate::scraper::r2::R2Client;

    let pool = &state.pool;
    let r2   = R2Client::from_env()?;

    LogRepository::insert(&app, pool, None, "INFO", "pab_import",
        &format!("Import started — {} file(s)", paths.len()),
    ).await.ok();

    // Pre-load all preset_ids that already have a PAB uploaded
    let existing_preset_ids: std::collections::HashSet<i64> = sqlx::query_scalar(
        "SELECT preset_id FROM scraper_preset_pabs",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect();

    let mut found   = 0usize;
    let mut patched = 0usize;
    let mut not_found_files: Vec<(String, Vec<u8>)> = Vec::new();

    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };

        let mut bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                LogRepository::insert(&app, pool, None, "ERR", "pab_import",
                    &format!("{}: read error — {}", filename, e),
                ).await.ok();
                continue;
            }
        };

        if bytes.len() > 0x44 {
            bytes[0x44] = 0xCC;
            patched += 1;
        }

        let Some(preset_id) = extract_preset_id(&filename) else {
            LogRepository::insert(&app, pool, None, "WARN", "pab_import",
                &format!("{}: no preset ID in filename", filename),
            ).await.ok();
            not_found_files.push((filename, bytes));
            continue;
        };

        match PresetRepository::get_with_class(pool, preset_id).await {
            Ok(Some((id, class_name))) => {
                let key     = format!("presets/{}/{}/{}", class_name, id, filename);
                let db_path = format!("/{}", key);

                if existing_preset_ids.contains(&id) {
                    LogRepository::insert(&app, pool, None, "INFO", "pab_import",
                        &format!("{}: preset {} already has a PAB, skipped", filename, id),
                    ).await.ok();
                    found += 1;
                } else {
                    match r2.upload(&key, bytes).await {
                        Ok(_) => {
                            PabRepository::insert(pool, id, &db_path).await.ok();
                            LogRepository::insert(&app, pool, None, "SYNC", "pab_import",
                                &format!("{}: uploaded → {}", filename, db_path),
                            ).await.ok();
                            found += 1;
                        }
                        Err(e) => {
                            LogRepository::insert(&app, pool, None, "ERR", "pab_import",
                                &format!("{}: R2 upload failed — {}", filename, e),
                            ).await.ok();
                        }
                    }
                }
            }
            _ => {
                LogRepository::insert(&app, pool, None, "WARN", "pab_import",
                    &format!("{}: preset {} not found in DB", filename, preset_id),
                ).await.ok();
                not_found_files.push((filename, bytes));
            }
        }
    }

    LogRepository::insert(&app, pool, None, "INFO", "pab_import",
        &format!("Import done — {} uploaded, {} not found", found, not_found_files.len()),
    ).await.ok();

    let not_found_count = not_found_files.len();
    let not_found_names: Vec<String> = not_found_files.iter().map(|(n, _)| n.clone()).collect();

    let zip_base64 = if not_found_files.is_empty() {
        None
    } else {
        let mut buf = Vec::new();
        {
            let mut zw = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, data) in &not_found_files {
                zw.start_file(name, opts)
                    .map_err(|e| AppError::Scrape(e.to_string()))?;
                zw.write_all(data)
                    .map_err(|e| AppError::Scrape(e.to_string()))?;
            }
            zw.finish().map_err(|e| AppError::Scrape(e.to_string()))?;
        }
        Some(base64::engine::general_purpose::STANDARD.encode(&buf))
    };

    Ok(ImportPabResult { found, patched, not_found_count, not_found_names, zip_base64 })
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
