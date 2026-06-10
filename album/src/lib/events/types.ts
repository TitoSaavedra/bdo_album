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

// ── Event → Payload map  (extend here to add new events) ─────

export interface RustEventMap {
  'db_ready':         DbReady;
  'preset_uploaded':  PresetUploaded;
  'listener_status':  ListenerStatus;
}

export type RustEventName = keyof RustEventMap;
