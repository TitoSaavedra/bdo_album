use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// ── Payloads ─────────────────────────────────────────────────
// Field names must match TypeScript interfaces in lib/events/types.ts.

#[derive(Serialize, Clone)]
pub struct DbReady {
    pub success: bool,
    pub error:   Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PresetUploaded {
    pub preset_id:   i64,
    pub class_id:    i32,
    pub image_1_url: Option<String>,
    pub image_2_url: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ListenerStatus {
    pub connected: bool,
}

// ── Event emitter ─────────────────────────────────────────────
// One method per event. Named after the event string the frontend listens to.
// Adding a new event = add payload struct above + method here.

pub struct Events;

impl Events {
    pub fn db_ready(app: &AppHandle, payload: DbReady) {
        app.emit("db_ready", payload).ok();
    }

    pub fn preset_uploaded(app: &AppHandle, payload: PresetUploaded) {
        app.emit("preset_uploaded", payload).ok();
    }

    pub fn listener_status(app: &AppHandle, payload: ListenerStatus) {
        app.emit("listener_status", payload).ok();
    }
}
