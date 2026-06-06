use sqlx::PgPool;

pub struct AppState {
    pub pool:          PgPool,
    pub r2_public_url: String,
}

impl AppState {
    pub fn new(pool: PgPool, r2_public_url: String) -> Self {
        Self { pool, r2_public_url }
    }
}
