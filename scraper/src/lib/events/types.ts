// ── Enums ────────────────────────────────────────────────────

export type ProgressStatus = 'processing' | 'metadata' | 'done';
export type ProgressType   = 'preset' | 'popular';
export type ScraperPhase  = 'fetch' | 'download' | 'upload';
export type ScraperStatus = 'idle' | 'running' | 'stopping' | 'done' | 'error' | 'cancelled';

export type DbErrorCode = 'docker_not_running' | 'env_var_missing';

export type SyncPhase =
  | 'fetching_presets'
  | 'starting_browser'
  | 'waiting_cf_clearance'
  | 'downloading_images'
  | 'fetching_and_downloading';

// Mirrors Rust `LogCode` (events.rs) — internally tagged on `code`, snake_case.
// Only the subset of LogEntry.msg that's a fixed template gets a code; free-form
// diagnostic messages (wrapped external errors) have `code: null` and only `msg`.
export type LogCode =
  | { code: 'verifying_chromium' }
  | { code: 'starting_playwright' }
  | { code: 'launching_chromium' }
  | { code: 'navigating_garmoth' }
  | { code: 'garmoth_loaded_waiting_cookie' }
  | { code: 'fetch_only_mode' }
  | { code: 'browser_starting' }
  | { code: 'browser_ready' }
  | { code: 'waiting_cf_clearance' }
  | { code: 'cf_clearance_obtained'; obtained: boolean }
  | { code: 'images_only_mode' }
  | { code: 'fetch_phase_started' }
  | { code: 'fetch_only_done'; processed: number; errors: number }
  | { code: 'fetch_done'; new: number; errors: number }
  | { code: 'images_done'; done: number; uploaded: number; errors: number }
  | { code: 'session_cancelled' }
  | { code: 'session_done'; session_id: number; elapsed: number; presets: number; images: number; errors: number }
  | { code: 'preloaded_ids'; count: number }
  | { code: 'skipping_db_preseed' }
  | { code: 'request_plan'; classes: number; days: number; regions: number; total: number }
  | { code: 'db_refresh'; added: number; total_seen: number }
  | { code: 'fetch_batch_progress'; remaining: number; total: number }
  | { code: 'class_results_update'; label: string; results: number; new: number; updated: number; skipped: number }
  | { code: 'class_results_fetch'; label: string; results: number; new: number; skipped: number }
  | { code: 'round_progress_update'; round: number; total_rounds: number; remaining: number; new: number; updated: number; skipped: number }
  | { code: 'round_progress_fetch'; round: number; total_rounds: number; remaining: number; new: number; skipped: number }
  | { code: 'class_done_update'; class: string; new: number; updated: number; skipped: number; errors: number }
  | { code: 'class_done_fetch'; class: string; new: number; skipped: number; errors: number }
  | { code: 'fetch_only_rounds_done'; rounds: number; processed: number; updated: number; errors: number }
  | { code: 'fetch_rounds_done'; rounds: number; presets: number; errors: number }
  | { code: 'img_page_failed_retry'; preset_id: number }
  | { code: 'img_upload_failed'; preset_id: number; image_num: number }
  | { code: 'img_not_found'; preset_id: number; image_num: number }
  | { code: 'images_uploaded'; preset_id: number; count: number }
  | { code: 'stats_update_failed'; preset_id: number }
  | { code: 'upsert_failed'; preset_id: number }
  | { code: 'insert_failed'; preset_id: number }
  | { code: 'no_pending_images_classes' }
  | { code: 'class_filter_round'; round: number; count: number; limit_per_class: number }
  | { code: 'fairness_skip'; names: string; min_dl: number; threshold: number }
  | { code: 'no_pending_images' }
  | { code: 'pending_round'; round: number; count: number }
  | { code: 'session_started'; session_id: number; mode: string; parallelism: number; classes: string; days: string; regions: string }
  | { code: 'import_started'; file_count: number }
  | { code: 'import_read_error'; filename: string }
  | { code: 'import_no_preset_id'; filename: string }
  | { code: 'import_already_has_pab'; filename: string; preset_id: number }
  | { code: 'import_uploaded'; filename: string; db_path: string }
  | { code: 'import_r2_upload_failed'; filename: string }
  | { code: 'import_preset_not_found'; filename: string; preset_id: number }
  | { code: 'import_done'; uploaded: number; not_found: number }
  | { code: 'db_connected_recovered'; recovered: number }
  | { code: 'db_connected_ready' };

// ── Payloads ─────────────────────────────────────────────────

export interface ScraperProgress {
  preset_id:     string;
  class_id:      number;
  class_name:    string;
  current:       number;
  total:         number;
  status:        ProgressStatus;
  progress_type: ProgressType;
}

export interface ScraperDone {
  total_fetched:  number;
  total_updated:  number;
  total_images:   number;
  total_uploaded: number;
  errors:         number;
  elapsed_secs:   number;
}

export interface ScraperError {
  message: string;
  phase:   ScraperPhase;
}

export interface FetchProgress {
  class_id:   number;
  class_name: string;
  fetched:    number;
  total:      number;
}

export interface ImageProgress {
  preset_id:  string;
  class_name: string;
  image_num:  1 | 2;
  done:       number;
  total:      number;
}

export interface UploadProgress {
  preset_id: string;
  image_url: string;
  done:      number;
  total:     number;
}

export interface ClassStatsUpdated {
  class_id:  number;
  fetched:   number;
  images:    number;
  errors:    number;
  skipped:   number;
}

export interface PresetSynced {
  preset_id:      string;
  class_id:       number;
  image_1_url:    string | null;
  image_2_url:    string | null;
  downloads:      number | null;
  views:          number | null;
  likes:          number | null;
  character_name: string | null;
}

export interface LogEntry {
  ts:     number;
  tag:    string;
  source: string;
  msg:    string;
  code:   LogCode | null;
}

export interface DbReady {
  success: boolean;
  error:   DbErrorCode | null;
}

export type AutoDownloadStatus =
  | { state: 'idle' }
  | { state: 'downloading'; preset_id: number }
  | { state: 'quota_exceeded'; used: number; limit: number };

// ── Event → Payload map  (extend here to add new events) ─────

export interface RustEventMap {
  // Lifecycle
  'scraper_started':    void;
  'scraper_done':       ScraperDone;
  'scraper_cancelled':  void;
  'scraper_error':      ScraperError;
  'scraper_fetch_done': void;

  // Progress
  'scraper_progress':  ScraperProgress;
  'fetch_progress':     FetchProgress;
  'image_progress':     ImageProgress;
  'upload_progress':    UploadProgress;

  // Data
  'class_stats_updated': ClassStatsUpdated;
  'preset_synced':       PresetSynced;

  // Infrastructure
  'db_ready':       DbReady;
  'log_entry':      LogEntry;
  'sync_loading':   SyncPhase;
  'auto_download_status': AutoDownloadStatus;
}

export type RustEventName = keyof RustEventMap;
