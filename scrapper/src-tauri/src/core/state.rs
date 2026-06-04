use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub pool:            PgPool,
    pub cf_clearance:    Mutex<String>,     // in memory, never persisted
    pub current_session: Mutex<Option<i64>>,
    pub cancel:          Arc<AtomicBool>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cf_clearance:    Mutex::new(String::new()),
            current_session: Mutex::new(None),
            cancel:          Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_cf_token(&self, token: String) {
        if let Ok(mut g) = self.cf_clearance.lock() { *g = token; }
    }

    pub fn get_cf_token(&self) -> String {
        self.cf_clearance.lock().map(|g| g.clone()).unwrap_or_default()
    }
}
