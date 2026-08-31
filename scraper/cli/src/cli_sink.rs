use std::sync::Arc;

use bdo_scraper_core::events::{
    AutoDownloadStatus, ClassStatsUpdated, FetchProgress, ImageProgress, LogEntry, PresetSynced,
    ScraperDone, ScraperError, ScraperProgress, Sink, SyncPhase, UploadProgress,
};

/// [`Sink`] implementation shared by both CLI binaries (`scrape`, `download_daemon`).
///
/// Unlike the GUI's `TauriSink`, there is no live listener to notify — nothing
/// on the server is watching for `scraper_progress`/`upload_progress`/etc. in
/// real time. Every one of those is therefore a no-op *except* `log_entry`,
/// which forwards to `tracing` so `journalctl -u bdo-scraper` shows readable
/// progress lines. Everything that actually matters for observability already
/// persists to Postgres (`scraper_sessions`, `scraper_logs`,
/// `scraper_class_stats`) regardless of which `Sink` is wired up — that's the
/// whole point of the `Sink` abstraction: the event-emitting side is optional,
/// the DB writes are not.
pub struct CliSink;

impl CliSink {
    pub fn new() -> Arc<dyn Sink> {
        Arc::new(Self)
    }
}

impl Sink for CliSink {
    fn scraper_started(&self) {}
    fn scraper_done(&self, _payload: ScraperDone) {}
    fn scraper_cancelled(&self) {}
    fn scraper_error(&self, _payload: ScraperError) {}

    fn scraper_progress(&self, _payload: ScraperProgress) {}
    fn fetch_progress(&self, _payload: FetchProgress) {}
    fn image_progress(&self, _payload: ImageProgress) {}
    fn upload_progress(&self, _payload: UploadProgress) {}

    fn class_stats_updated(&self, _payload: ClassStatsUpdated) {}
    fn preset_synced(&self, _payload: PresetSynced) {}

    fn log_entry(&self, payload: LogEntry) {
        match payload.tag.as_str() {
            "ERR"  => tracing::error!(source = %payload.source, "{}", payload.msg),
            "WARN" => tracing::warn!(source = %payload.source, "{}", payload.msg),
            _      => tracing::info!(source = %payload.source, "{}", payload.msg),
        }
    }
    fn sync_loading(&self, _phase: SyncPhase) {}
    fn fetch_done(&self) {}
    fn auto_download_status(&self, _payload: AutoDownloadStatus) {}
}
