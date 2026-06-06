use sqlx::PgPool;

use crate::core::errors::Result;

// The album only reads from the DB — migrations are owned by the scrapper.
pub async fn init(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}
