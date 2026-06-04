use serde::Serialize;
use tauri::{AppHandle, Emitter};

// ── Enums ────────────────────────────────────────────────────
// Serialized as lowercase strings — must match TypeScript union types exactly.

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    Processing,
    Metadata,
    Done,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProgressType {
    Preset,
    Popular,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ScrapperPhase {
    Fetch,
    Download,
    Upload,
}

// ── Payloads ─────────────────────────────────────────────────
// Field names must match TypeScript interfaces in lib/events/types.ts.

#[derive(Serialize, Clone)]
pub struct ScrapperProgress {
    pub preset_id:     String,
    pub class_id:      u32,
    pub class_name:    String,
    pub current:       usize,
    pub total:         usize,
    pub status:        ProgressStatus,
    pub message:       String,
    pub progress_type: ProgressType,
}

#[derive(Serialize, Clone)]
pub struct ScrapperDone {
    pub total_fetched:  usize,
    pub total_images:   usize,
    pub total_uploaded: usize,
    pub errors:         usize,
    pub elapsed_secs:   u64,
}

#[derive(Serialize, Clone)]
pub struct ScrapperError {
    pub message: String,
    pub phase:   ScrapperPhase,
}

#[derive(Serialize, Clone)]
pub struct FetchProgress {
    pub class_id:   u32,
    pub class_name: String,
    pub fetched:    usize,
    pub total:      usize,
}

#[derive(Serialize, Clone)]
pub struct ImageProgress {
    pub preset_id:  String,
    pub class_name: String,
    pub image_num:  u8,      // 1 | 2
    pub done:       usize,
    pub total:      usize,
}

#[derive(Serialize, Clone)]
pub struct UploadProgress {
    pub preset_id: String,
    pub image_url: String,
    pub done:      usize,
    pub total:     usize,
}

#[derive(Serialize, Clone)]
pub struct ClassStatsUpdated {
    pub class_id: u32,
    pub fetched:  usize,
    pub images:   usize,
    pub errors:   usize,
}

#[derive(Serialize, Clone)]
pub struct PresetSynced {
    pub preset_id:   String,
    pub class_id:    u32,
    pub image_1_url: Option<String>,
    pub image_2_url: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub ts:     i64,
    pub tag:    String,
    pub source: String,
    pub msg:    String,
}

#[derive(Serialize, Clone)]
pub struct DbReady {
    pub success: bool,
    pub error:   Option<String>,
}

// ── Event emitter ─────────────────────────────────────────────
// One method per event. Named after the event string the frontend listens to.
// Adding a new event = add payload struct above + method here.

pub struct Events;

impl Events {
    // Lifecycle
    pub fn scrapper_started(app: &AppHandle) {
        app.emit("scrapper_started", ()).ok();
    }

    pub fn scrapper_done(app: &AppHandle, payload: ScrapperDone) {
        app.emit("scrapper_done", payload).ok();
    }

    pub fn scrapper_cancelled(app: &AppHandle) {
        app.emit("scrapper_cancelled", ()).ok();
    }

    pub fn scrapper_error(app: &AppHandle, payload: ScrapperError) {
        app.emit("scrapper_error", payload).ok();
    }

    // Progress
    pub fn scrapper_progress(app: &AppHandle, payload: ScrapperProgress) {
        app.emit("scrapper_progress", payload).ok();
    }

    pub fn fetch_progress(app: &AppHandle, payload: FetchProgress) {
        app.emit("fetch_progress", payload).ok();
    }

    pub fn image_progress(app: &AppHandle, payload: ImageProgress) {
        app.emit("image_progress", payload).ok();
    }

    pub fn upload_progress(app: &AppHandle, payload: UploadProgress) {
        app.emit("upload_progress", payload).ok();
    }

    // Data
    pub fn class_stats_updated(app: &AppHandle, payload: ClassStatsUpdated) {
        app.emit("class_stats_updated", payload).ok();
    }

    pub fn preset_synced(app: &AppHandle, payload: PresetSynced) {
        app.emit("preset_synced", payload).ok();
    }

    // Infrastructure
    pub fn db_ready(app: &AppHandle, payload: DbReady) {
        app.emit("db_ready", payload).ok();
    }

    pub fn log_entry(app: &AppHandle, payload: LogEntry) {
        app.emit("log_entry", payload).ok();
    }

    pub fn sync_loading(app: &AppHandle, msg: &str) {
        app.emit("sync_loading", msg).ok();
    }
}
