use tauri::State;

use crate::core::state::AppState;
use crate::beauty::service::BeautyService;
use crate::db::repositories::class_repo::ClassRow;
use crate::db::repositories::preset_repo::PresetRow;

// ── Classes ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_classes(state: State<'_, AppState>) -> Result<Vec<ClassRow>, String> {
    BeautyService::get_classes(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_class_favorites(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    BeautyService::get_class_favorites(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_class_favorite(
    class_name:  String,
    is_favorite: bool,
    state:       State<'_, AppState>,
) -> Result<(), String> {
    BeautyService::set_class_favorite(&state.pool, &class_name, is_favorite)
        .await
        .map_err(|e| e.to_string())
}

// ── Presets ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_presets(
    class_name: String,
    offset:     Option<i64>,
    limit:      Option<i64>,
    sort_by:    Option<String>,
    search:     Option<String>,
    state:      State<'_, AppState>,
) -> Result<Vec<PresetRow>, String> {
    BeautyService::get_presets(
        &state.pool,
        &class_name,
        offset.unwrap_or(0),
        limit.unwrap_or(50),
        sort_by.as_deref().unwrap_or("downloads"),
        search.as_deref().unwrap_or(""),
        &state.r2_public_url,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discard_preset(
    preset_id: String,
    state:     State<'_, AppState>,
) -> Result<(), String> {
    let id: i64 = preset_id.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    BeautyService::discard_preset(&state.pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_wanted(
    preset_id: String,
    state:     State<'_, AppState>,
) -> Result<bool, String> {
    let id: i64 = preset_id.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    BeautyService::toggle_wanted(&state.pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wanted(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    BeautyService::get_wanted_ids(&state.pool).await.map_err(|e| e.to_string())
}
