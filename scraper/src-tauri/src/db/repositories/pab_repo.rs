use sqlx::PgPool;

use crate::core::errors::Result;

pub struct PabRepository;

impl PabRepository {
    pub async fn insert(pool: &PgPool, preset_id: i64, url: &str) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scraper_preset_pabs (preset_id, url) VALUES ($1, $2) RETURNING id",
        )
        .bind(preset_id)
        .bind(url)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }
}
