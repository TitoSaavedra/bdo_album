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
pub enum ScraperPhase {
    Fetch,
    Download,
    Upload,
}

// ── Payloads ─────────────────────────────────────────────────
// Field names must match TypeScript interfaces in lib/events/types.ts.

#[derive(Serialize, Clone)]
pub struct ScraperProgress {
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
pub struct ScraperDone {
    pub total_fetched:  usize,
    pub total_updated:  usize,
    pub total_images:   usize,
    pub total_uploaded: usize,
    pub errors:         usize,
    pub elapsed_secs:   u64,
}

#[derive(Serialize, Clone)]
pub struct ScraperError {
    pub message: String,
    pub phase:   ScraperPhase,
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
    pub skipped:  usize,
}

#[derive(Serialize, Clone)]
pub struct PresetSynced {
    pub preset_id:      String,
    pub class_id:       u32,
    pub image_1_url:    Option<String>,
    pub image_2_url:    Option<String>,
    pub downloads:      Option<i64>,
    pub views:          Option<i64>,
    pub likes:          Option<i64>,
    pub character_name: Option<String>,
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
    pub fn scraper_started(app: &AppHandle) {
        app.emit("scraper_started", ()).ok();
    }

    pub fn scraper_done(app: &AppHandle, payload: ScraperDone) {
        app.emit("scraper_done", payload).ok();
    }

    pub fn scraper_cancelled(app: &AppHandle) {
        app.emit("scraper_cancelled", ()).ok();
    }

    pub fn scraper_error(app: &AppHandle, payload: ScraperError) {
        app.emit("scraper_error", payload).ok();
    }

    // Progress
    pub fn scraper_progress(app: &AppHandle, payload: ScraperProgress) {
        app.emit("scraper_progress", payload).ok();
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

    pub fn fetch_done(app: &AppHandle) {
        app.emit("scraper_fetch_done", ()).ok();
    }
}
