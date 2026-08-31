use tauri::{AppHandle, Emitter};

use bdo_scraper_core::events::{
    AutoDownloadStatus, ClassStatsUpdated, FetchProgress, ImageProgress, LogEntry, PresetSynced,
    ScraperDone, ScraperError, ScraperProgress, Sink, SyncPhase, UploadProgress,
};

/// [`Sink`] implementation for the desktop GUI — each method's body is
/// exactly what `events::Events::*` used to do before those methods moved
/// into `bdo_scraper_core::events::Sink`: a best-effort `app.emit(...)`.
pub struct TauriSink(pub AppHandle);

impl Sink for TauriSink {
    fn scraper_started(&self) {
        self.0.emit("scraper_started", ()).ok();
    }

    fn scraper_done(&self, payload: ScraperDone) {
        self.0.emit("scraper_done", payload).ok();
    }

    fn scraper_cancelled(&self) {
        self.0.emit("scraper_cancelled", ()).ok();
    }

    fn scraper_error(&self, payload: ScraperError) {
        self.0.emit("scraper_error", payload).ok();
    }

    fn scraper_progress(&self, payload: ScraperProgress) {
        self.0.emit("scraper_progress", payload).ok();
    }

    fn fetch_progress(&self, payload: FetchProgress) {
        self.0.emit("fetch_progress", payload).ok();
    }

    fn image_progress(&self, payload: ImageProgress) {
        self.0.emit("image_progress", payload).ok();
    }

    fn upload_progress(&self, payload: UploadProgress) {
        self.0.emit("upload_progress", payload).ok();
    }

    fn class_stats_updated(&self, payload: ClassStatsUpdated) {
        self.0.emit("class_stats_updated", payload).ok();
    }

    fn preset_synced(&self, payload: PresetSynced) {
        self.0.emit("preset_synced", payload).ok();
    }

    fn log_entry(&self, payload: LogEntry) {
        self.0.emit("log_entry", payload).ok();
    }

    fn sync_loading(&self, phase: SyncPhase) {
        self.0.emit("sync_loading", phase).ok();
    }

    fn fetch_done(&self) {
        self.0.emit("scraper_fetch_done", ()).ok();
    }

    fn auto_download_status(&self, payload: AutoDownloadStatus) {
        self.0.emit("auto_download_status", payload).ok();
    }
}
