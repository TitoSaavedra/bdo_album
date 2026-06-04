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
            let app_h = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let dotenv_result = dotenvy::dotenv();
                eprintln!("[startup] dotenv: {:?}", dotenv_result);

                let db_url = match std::env::var("DATABASE_URL") {
                    Ok(url) => {
                        eprintln!("[startup] DATABASE_URL found: {}", url);
                        url
                    }
                    Err(e) => {
                        eprintln!("[startup] DATABASE_URL missing: {}", e);
                        Events::db_ready(&app_h, DbReady {
                            success: false,
                            error: Some(format!("DATABASE_URL not set: {}", e)),
                        });
                        return;
                    }
                };

                eprintln!("[startup] connecting to DB...");
                match db::pool::init(&db_url).await {
                    Ok(pool) => {
                        eprintln!("[startup] DB connected OK");
                        // Save startup log before moving pool into AppState
                        db::repositories::log_repo::LogRepository::insert(
                            &app_h, &pool, None, "INFO", "startup",
                            "Database connected. Ready to scrape.",
                        ).await.ok();
                        app_h.manage(AppState::new(pool));
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        Events::db_ready(&app_h, DbReady { success: true, error: None });
                    }
                    Err(e) => {
                        eprintln!("[startup] DB error: {}", e);
                        Events::db_ready(&app_h, DbReady {
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scraper::commands::get_db_status,
            scraper::commands::get_classes,
            scraper::commands::run_scraper,
            scraper::commands::cancel_scraper,
            scraper::commands::get_sessions,
            scraper::commands::get_class_stats_cmd,
            scraper::commands::get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
