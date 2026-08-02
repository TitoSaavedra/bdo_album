mod app;
mod beauty;
mod core;
mod db;
mod face_grid;

use std::sync::atomic::Ordering;
use core::events::{DbErrorCode, DbReady, Events, ListenerStatus, PresetUploaded};
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
                        Err(_) => {
                            set_listener_state(&app, false);
                            break;
                        }
                    }
                }
            }
            Err(_) => {
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
                    Err(_e) => {
                        Events::db_ready(&app_h, DbReady {
                            success: false,
                            error: Some(DbErrorCode::EnvVarMissing),
                        });
                        return;
                    }
                };

                let r2_public_url = std::env::var("R2_PUBLIC_URL")
                    .unwrap_or_default()
                    .trim_end_matches('/')
                    .to_string();

                let mut attempts = 0u32;
                let mut warned = false;
                let pool = loop {
                    match core::db::init(&db_url).await {
                        Ok(p) => break p,
                        Err(_) => {
                            attempts += 1;
                            if attempts == 5 && !warned {
                                warned = true;
                                Events::db_ready(&app_h, DbReady {
                                    success: false,
                                    error: Some(DbErrorCode::DockerNotRunning),
                                });
                            }
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        }
                    }
                };
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
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
            beauty::commands::get_wanted_pab_urls,
            beauty::commands::get_wanted_presets,
            beauty::commands::export_to_bdo,
            beauty::commands::get_class_search_counts,
            face_grid::commands::scan_bdo_accounts,
            face_grid::commands::list_face_textures,
            face_grid::commands::save_face_grid,
            face_grid::commands::get_face_grids,
            face_grid::commands::get_face_grid_slots,
            face_grid::commands::apply_face_grid,
            face_grid::commands::delete_face_grid,
            face_grid::commands::overwrite_face_grid,
            face_grid::commands::get_character_faces,
            face_grid::commands::save_face_to_disk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
