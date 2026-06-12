use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub pool:            PgPool,
    pub current_session: Mutex<Option<i64>>,
    pub cancel:          Arc<AtomicBool>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            current_session: Mutex::new(None),
            cancel:          Arc::new(AtomicBool::new(false)),
        }
    }
}
