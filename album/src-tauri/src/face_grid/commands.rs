use tauri::State;

use crate::core::state::AppState;
use crate::db::repositories::face_grid_repo::{FaceGridRow, FaceGridSlotRow};
use crate::face_grid::service::{BdoAccount, FaceGridService, FaceTextureEntry, SlotAssignment};

#[tauri::command]
pub async fn scan_bdo_accounts() -> Result<Vec<BdoAccount>, String> {
    FaceGridService::scan_bdo_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_face_textures() -> Result<Vec<FaceTextureEntry>, String> {
    FaceGridService::list_face_textures().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_face_to_slot(
    character_no: String,
    image_url:    String,
) -> Result<(), String> {
    FaceGridService::apply_face_to_slot(&character_no, &image_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_face_grid(
    name:       String,
    account_id: String,
    slots:      Vec<SlotAssignment>,
    state:      State<'_, AppState>,
) -> Result<FaceGridRow, String> {
    FaceGridService::save_face_grid(
        &state.pool,
        state.r2_client.as_ref(),
        &state.r2_public_url,
        &name,
        &account_id,
        &slots,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_face_grids(state: State<'_, AppState>) -> Result<Vec<FaceGridRow>, String> {
    FaceGridService::get_face_grids(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_face_grid_slots(
    grid_id: i64,
    state:   State<'_, AppState>,
) -> Result<Vec<FaceGridSlotRow>, String> {
    FaceGridService::get_face_grid_slots(&state.pool, grid_id, &state.r2_public_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_face_grid(
    grid_id: i64,
    state:   State<'_, AppState>,
) -> Result<(), String> {
    FaceGridService::apply_face_grid(&state.pool, grid_id, &state.r2_public_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_face_grid(
    grid_id: i64,
    state:   State<'_, AppState>,
) -> Result<(), String> {
    FaceGridService::delete_face_grid(&state.pool, grid_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_character_face(
    character_no: String,
    image_b64:    String,
    state:        State<'_, AppState>,
) -> Result<String, String> {
    FaceGridService::upload_character_face(&state.pool, state.r2_client.as_ref(), &character_no, &image_b64)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_character_faces(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    FaceGridService::get_character_faces(&state.pool)
        .await
        .map_err(|e| e.to_string())
}
