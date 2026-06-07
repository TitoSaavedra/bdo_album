mod app;
mod beauty;
mod core;
mod db;

use core::events::{DbReady, Events};
use core::state::AppState;
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
                    Ok(url) => url,
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
                let r2_public_url = std::env::var("R2_PUBLIC_URL")
                    .unwrap_or_default()
                    .trim_end_matches('/')
                    .to_string();

                let pool = loop {
                    match core::db::init(&db_url).await {
                        Ok(p) => break p,
                        Err(e) => {
                            eprintln!("[startup] DB error: {}, retrying in 3s...", e);
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        }
                    }
                };
                eprintln!("[startup] DB connected OK");
                app_h.manage(AppState::new(pool, r2_public_url));
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                Events::db_ready(&app_h, DbReady { success: true, error: None });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::commands::is_db_ready,
            app::commands::open_url,
            beauty::commands::get_classes,
            beauty::commands::get_class_favorites,
            beauty::commands::set_class_favorite,
            beauty::commands::get_presets,
            beauty::commands::discard_preset,
            beauty::commands::toggle_wanted,
            beauty::commands::get_wanted,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
