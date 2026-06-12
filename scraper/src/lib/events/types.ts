// ── Enums ────────────────────────────────────────────────────

export type ProgressStatus = 'processing' | 'metadata' | 'done';
export type ProgressType   = 'preset' | 'popular';
export type ScraperPhase  = 'fetch' | 'download' | 'upload';
export type ScraperStatus = 'idle' | 'running' | 'stopping' | 'done' | 'error' | 'cancelled';

// ── Payloads ─────────────────────────────────────────────────

export interface ScraperProgress {
  preset_id:     string;
  class_id:      number;
  class_name:    string;
  current:       number;
  total:         number;
  status:        ProgressStatus;
  message:       string;
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
}

export interface DbReady {
  success: boolean;
  error:   string | null;
}

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
  'sync_loading':   string;
}

export type RustEventName = keyof RustEventMap;
