pub mod events;
mod app;
mod core;
mod db;
mod scraper;

use core::state::AppState;
use events::{DbReady, Events};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(win) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = win.primary_monitor() {
                    let s = monitor.size();
                    let w = (s.width as f64 * 0.60) as u32;
                    let h = (s.height as f64 * 0.80) as u32;
                    let x = (s.width as f64 * 0.20) as i32;
                    let y = (s.height as f64 * 0.10) as i32;
                    let _ = win.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: w, height: h }));
                    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                }
                let _ = win.show();
            }
            let app_h = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(exe) = std::env::current_exe() {
                    if let Some(dir) = exe.parent() {
                        dotenvy::from_path(dir.join(".env")).ok();
                    }
                }
                dotenvy::dotenv().ok();

                let db_url = match std::env::var("DATABASE_URL") {
                    Ok(url) => url,
                    Err(e) => {
                        Events::db_ready(&app_h, DbReady {
                            success: false,
                            error: Some(format!("DATABASE_URL not set: {}", e)),
                        });
                        return;
                    }
                };

                let mut attempts = 0u32;
                let mut warned = false;
                let pool = loop {
                    match db::pool::init(&db_url).await {
                        Ok(p) => break p,
                        Err(_) => {
                            attempts += 1;
                            if attempts == 5 && !warned {
                                warned = true;
                                Events::db_ready(&app_h, DbReady {
                                    success: false,
                                    error: Some("No se pudo conectar a la base de datos. ¿Está Docker corriendo? (docker compose up -d)".to_string()),
                                });
                            }
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        }
                    }
                };
                db::repositories::log_repo::LogRepository::prune(&pool, 30).await.ok();
                let recovered = db::repositories::session_repo::SessionRepository::recover_interrupted(&pool).await.unwrap_or(0);
                db::repositories::log_repo::LogRepository::insert(
                    &app_h, &pool, None, "INFO", "startup",
                    &if recovered > 0 {
                        format!("Database connected. {} interrupted session(s) recovered.", recovered)
                    } else {
                        "Database connected. Ready to scrape.".to_string()
                    },
                ).await.ok();
                app_h.manage(AppState::new(pool));
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                Events::db_ready(&app_h, DbReady { success: true, error: None });
            });
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            app::commands::open_url,
            scraper::commands::get_db_status,
            scraper::commands::get_classes,
            scraper::commands::run_scraper,
            scraper::commands::cancel_scraper,
            scraper::commands::get_sessions,
            scraper::commands::get_session_totals,
            scraper::commands::get_class_stats_cmd,
            scraper::commands::get_preset_stats,
            scraper::commands::get_logs,
            scraper::commands::import_pab_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
