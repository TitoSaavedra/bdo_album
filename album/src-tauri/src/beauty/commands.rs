use serde::Serialize;
use tauri::{Manager, State};

use crate::core::state::AppState;
use crate::beauty::service::BeautyService;
use crate::db::repositories::class_repo::ClassRow;
use crate::db::repositories::preset_repo::PresetRow;

#[derive(Serialize)]
pub struct ClassCount {
    pub class_id: i32,
    pub count:    i64,
}

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
    region:     Option<String>,
    days:       Option<String>,
    state:      State<'_, AppState>,
) -> Result<Vec<PresetRow>, String> {
    BeautyService::get_presets(
        &state.pool,
        &class_name,
        offset.unwrap_or(0),
        limit.unwrap_or(50),
        sort_by.as_deref().unwrap_or("downloads"),
        search.as_deref().unwrap_or(""),
        region.as_deref(),
        days.as_deref(),
        &state.r2_public_url,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_preset(
    preset_id: String,
    state:     State<'_, AppState>,
) -> Result<Option<PresetRow>, String> {
    let id: i64 = preset_id.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    BeautyService::get_preset_by_id(&state.pool, id, &state.r2_public_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_regions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    BeautyService::get_regions(&state.pool).await.map_err(|e| e.to_string())
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

#[tauri::command]
pub async fn get_wanted_pab_urls(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    BeautyService::get_wanted_pab_urls(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wanted_presets(state: State<'_, AppState>) -> Result<Vec<PresetRow>, String> {
    BeautyService::get_wanted_presets(&state.pool, &state.r2_public_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn queue_auto_download(
    preset_ids: Vec<String>,
    state:      State<'_, AppState>,
) -> Result<(), String> {
    let ids: Vec<i64> = preset_ids.iter()
        .map(|s| s.parse::<i64>().map_err(|e| e.to_string()))
        .collect::<Result<_, String>>()?;
    BeautyService::queue_auto_download(&state.pool, &ids).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_to_bdo(
    pab_url: String,
    app:     tauri::AppHandle,
) -> Result<(), String> {
    let docs = app.path().document_dir().map_err(|e| e.to_string())?;
    let customization = docs.join("Black Desert").join("Customization");

    if !customization.exists() {
        return Err("Black Desert Customization folder not found. Is the game installed?".to_string());
    }

    // Send all files to the Recycle Bin
    for entry in std::fs::read_dir(&customization).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() {
            trash::delete(&path).map_err(|e| e.to_string())?;
        }
    }

    // Extract filename from URL (strip query params)
    let filename = pab_url
        .split('/')
        .last()
        .unwrap_or("preset.pab")
        .split('?')
        .next()
        .unwrap_or("preset.pab")
        .to_string();

    // Download PAB bytes — R2 holds the pristine original, so leaving it
    // editable in-game happens here, right before it's actually used.
    let response = reqwest::get(&pab_url).await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
    let bytes = BeautyService::patch_editable(bytes);

    // Write to Customization folder
    std::fs::write(customization.join(&filename), &bytes).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_class_search_counts(
    search: Option<String>,
    region: Option<String>,
    days:   Option<String>,
    state:  State<'_, AppState>,
) -> Result<Vec<ClassCount>, String> {
    let search = search.as_deref().unwrap_or("");
    let region = region.as_deref().filter(|s| !s.is_empty());
    let counts = BeautyService::get_class_search_counts(&state.pool, search, region, days.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(counts.into_iter().map(|(class_id, count)| ClassCount { class_id, count }).collect())
}
