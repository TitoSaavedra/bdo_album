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

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DbErrorCode {
    DockerNotRunning,
    EnvVarMissing,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    FetchingPresets,
    StartingBrowser,
    WaitingCfClearance,
    DownloadingImages,
    FetchingAndDownloading,
}

/// Structured code for a subset of `LogEntry.msg` — lets the frontend render a
/// localized sentence for the live "Logs" view. `scraper_logs.msg` in the DB always
/// keeps the full English text (built at the call site) as the source of truth;
/// this is only attached to the live Tauri event, never persisted. Messages that
/// wrap an arbitrary underlying error (browser/network/db failures) intentionally
/// have no code — there is no way to translate free-form diagnostic text.
#[derive(Serialize, Clone)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum LogCode {
    VerifyingChromium,
    StartingPlaywright,
    LaunchingChromium,
    NavigatingGarmoth,
    GarmothLoadedWaitingCookie,
    FetchOnlyMode,
    BrowserStarting,
    BrowserReady,
    WaitingCfClearance,
    CfClearanceObtained { obtained: bool },
    ImagesOnlyMode,
    FetchPhaseStarted,
    FetchOnlyDone { processed: i64, errors: i64 },
    FetchDone { new: i64, errors: i64 },
    ImagesDone { done: i64, uploaded: i64, errors: i64 },
    SessionCancelled,
    SessionDone { session_id: i64, elapsed: i64, presets: i64, images: i64, errors: i64 },
    PreloadedIds { count: i64 },
    SkippingDbPreseed,
    RequestPlan { classes: i64, days: i64, regions: i64, total: i64 },
    DbRefresh { added: i64, total_seen: i64 },
    FetchBatchProgress { remaining: i64, total: i64 },
    ClassResultsUpdate { label: String, results: i64, new: i64, updated: i64, skipped: i64 },
    ClassResultsFetch { label: String, results: i64, new: i64, skipped: i64 },
    RoundProgressUpdate { round: i64, total_rounds: i64, remaining: i64, new: i64, updated: i64, skipped: i64 },
    RoundProgressFetch { round: i64, total_rounds: i64, remaining: i64, new: i64, skipped: i64 },
    ClassDoneUpdate { class: String, new: i64, updated: i64, skipped: i64, errors: i64 },
    ClassDoneFetch { class: String, new: i64, skipped: i64, errors: i64 },
    FetchOnlyRoundsDone { rounds: i64, processed: i64, updated: i64, errors: i64 },
    FetchRoundsDone { rounds: i64, presets: i64, errors: i64 },
    ImgPageFailedRetry { preset_id: i64 },
    ImgUploadFailed { preset_id: i64, image_num: u8 },
    ImgNotFound { preset_id: i64, image_num: u8 },
    ImagesUploaded { preset_id: i64, count: i64 },
    StatsUpdateFailed { preset_id: i64 },
    UpsertFailed { preset_id: i64 },
    InsertFailed { preset_id: i64 },
    NoPendingImagesClasses,
    ClassFilterRound { round: i64, count: i64, limit_per_class: i64 },
    FairnessSkip { names: String, min_dl: i64, threshold: i64 },
    NoPendingImages,
    PendingRound { round: i64, count: i64 },
    SessionStarted { session_id: i64, mode: String, parallelism: i64, classes: String, days: String, regions: String },
    ImportStarted { file_count: i64 },
    ImportReadError { filename: String },
    ImportNoPresetId { filename: String },
    ImportAlreadyHasPab { filename: String, preset_id: i64 },
    ImportUploaded { filename: String, db_path: String },
    ImportR2UploadFailed { filename: String },
    ImportPresetNotFound { filename: String, preset_id: i64 },
    ImportDone { uploaded: i64, not_found: i64 },
    DbConnectedRecovered { recovered: i64 },
    DbConnectedReady,
    RepairPabStarted { preset_id: i64 },
    RepairPabUploaded { preset_id: i64, db_path: String },
    RepairPabFailed { preset_id: i64 },
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
    pub code:   Option<LogCode>,
}

#[derive(Serialize, Clone)]
pub struct DbReady {
    pub success: bool,
    pub error:   Option<DbErrorCode>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AutoDownloadStatus {
    Idle,
    Downloading { preset_id: i64 },
    QuotaExceeded { used: u32, limit: u32 },
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

    pub fn sync_loading(app: &AppHandle, phase: SyncPhase) {
        app.emit("sync_loading", phase).ok();
    }

    pub fn fetch_done(app: &AppHandle) {
        app.emit("scraper_fetch_done", ()).ok();
    }

    pub fn auto_download_status(app: &AppHandle, payload: AutoDownloadStatus) {
        app.emit("auto_download_status", payload).ok();
    }
}
