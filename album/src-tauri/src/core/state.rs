use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use sqlx::PgPool;

use crate::core::r2::R2Client;

pub struct AppState {
    pub pool:               PgPool,
    pub r2_public_url:      String,
    pub r2_client:          Option<R2Client>,
    pub listener_connected: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(pool: PgPool, r2_public_url: String, r2_client: Option<R2Client>) -> Self {
        Self {
            pool,
            r2_public_url,
            r2_client,
            listener_connected: Arc::new(AtomicBool::new(false)),
        }
    }
}
