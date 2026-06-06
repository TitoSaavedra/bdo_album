use serde::Serialize;
use tauri::{AppHandle, Emitter};

// ── Payloads ─────────────────────────────────────────────────
// Field names must match TypeScript interfaces in lib/events/types.ts.

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
    pub fn db_ready(app: &AppHandle, payload: DbReady) {
        app.emit("db_ready", payload).ok();
    }
}
