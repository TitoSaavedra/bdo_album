use serde::Serialize;
use tauri::{AppHandle, Emitter};

// `DbErrorCode`/`DbReady`/`Events::db_ready` are the only events left in this
// crate — everything else (`Sink`, its payload structs, `LogCode`, etc.) moved
// to `bdo_scraper_core::events` as part of extracting the headless CLI. These
// three stay here because they're purely about the Tauri window's own startup
// sequence (`lib.rs`'s `setup` closure) and have no meaning outside a GUI.

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DbErrorCode {
    DockerNotRunning,
    EnvVarMissing,
}

#[derive(Serialize, Clone)]
pub struct DbReady {
    pub success: bool,
    pub error:   Option<DbErrorCode>,
}

pub struct Events;

impl Events {
    pub fn db_ready(app: &AppHandle, payload: DbReady) {
        app.emit("db_ready", payload).ok();
    }
}
