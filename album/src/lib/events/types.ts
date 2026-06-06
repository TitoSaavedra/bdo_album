// ── Payloads ─────────────────────────────────────────────────
// Must mirror Rust structs in core/events.rs exactly.

export interface DbReady {
  success: boolean;
  error:   string | null;
}

// ── Event → Payload map  (extend here to add new events) ─────

export interface RustEventMap {
  'db_ready': DbReady;
}

export type RustEventName = keyof RustEventMap;
