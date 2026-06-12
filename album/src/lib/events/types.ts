// ── Payloads ─────────────────────────────────────────────────
// Must mirror Rust structs in core/events.rs exactly.

export interface DbReady {
  success: boolean;
  error:   string | null;
}

export interface PresetUploaded {
  preset_id:   number;
  class_id:    number;
  image_1_url: string | null;
  image_2_url: string | null;
}

export interface ListenerStatus {
  connected: boolean;
}

// ── Face Grid ─────────────────────────────────────────────────

export interface FaceGridProgress {
  current:      number;
  total:        number;
  character_no: string;
}

// ── Event → Payload map  (extend here to add new events) ─────

export interface RustEventMap {
  'db_ready':           DbReady;
  'preset_uploaded':    PresetUploaded;
  'listener_status':    ListenerStatus;
  'face_grid_progress': FaceGridProgress;
  'face_apply_done':    string;
}

export type RustEventName = keyof RustEventMap;
