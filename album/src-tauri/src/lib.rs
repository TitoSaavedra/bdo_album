mod app;
mod beauty;
mod core;
mod db;
mod face_grid;

use std::sync::atomic::Ordering;
use core::events::{DbReady, Events, ListenerStatus, PresetUploaded};
use core::state::AppState;
use sqlx::postgres::PgListener;
use tauri::{AppHandle, Manager};

fn set_listener_state(app: &AppHandle, connected: bool) {
    if let Some(state) = app.try_state::<AppState>() {
        state.listener_connected.store(connected, Ordering::Relaxed);
    }
    Events::listener_status(app, ListenerStatus { connected });
}

async fn preset_listen_loop(app: AppHandle, db_url: String) {
    loop {
        match PgListener::connect(&db_url).await {
            Ok(mut listener) => {
                if listener.listen("preset_uploaded").await.is_err() {
                    set_listener_state(&app, false);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
                set_listener_state(&app, true);
                loop {
                    match listener.recv().await {
                        Ok(notif) => {
                            if let Ok(payload) = serde_json::from_str::<PresetUploaded>(notif.payload()) {
                                Events::preset_uploaded(&app, payload);
                            }
                        }
                        Err(e) => {
                            eprintln!("[listen] recv error: {}, reconnecting...", e);
                            set_listener_state(&app, false);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[listen] PgListener connect error: {}", e);
                set_listener_state(&app, false);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

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
                let r2_client = core::r2::R2Client::from_env().ok();
                app_h.manage(AppState::new(pool, r2_public_url, r2_client));

                let listener_url = db_url.clone();
                let app_h2 = app_h.clone();
                tauri::async_runtime::spawn(async move {
                    preset_listen_loop(app_h2, listener_url).await;
                });

                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                Events::db_ready(&app_h, DbReady { success: true, error: None });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::commands::is_db_ready,
            app::commands::is_listener_connected,
            app::commands::open_url,
            beauty::commands::get_classes,
            beauty::commands::get_class_favorites,
            beauty::commands::set_class_favorite,
            beauty::commands::get_presets,
            beauty::commands::get_preset,
            beauty::commands::get_regions,
            beauty::commands::discard_preset,
            beauty::commands::toggle_wanted,
            beauty::commands::get_wanted,
            face_grid::commands::scan_bdo_accounts,
            face_grid::commands::list_face_textures,
            face_grid::commands::apply_face_to_slot,
            face_grid::commands::save_face_grid,
            face_grid::commands::get_face_grids,
            face_grid::commands::get_face_grid_slots,
            face_grid::commands::apply_face_grid,
            face_grid::commands::delete_face_grid,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
