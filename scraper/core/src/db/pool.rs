use sqlx::PgPool;

use crate::errors::Result;

// Migrations are applied manually via `sqlx migrate run` (see README), not on
// every app launch — auto-migrating on startup was fragile (a checksum drift
// from line-ending changes silently blocked every connection attempt) and
// coupled schema changes to whichever app happened to start first.
pub async fn init(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}
