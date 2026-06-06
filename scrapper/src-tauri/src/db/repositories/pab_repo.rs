use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(sqlx::FromRow)]
pub struct PabRow {
    pub id:        i64,
    pub preset_id: i64,
    pub url:       String,
}

pub struct PabRepository;

impl PabRepository {
    pub async fn insert(pool: &PgPool, preset_id: i64, url: &str) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scrapper_preset_pabs (preset_id, url) VALUES ($1, $2) RETURNING id",
        )
        .bind(preset_id)
        .bind(url)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }

    pub async fn get_by_preset(pool: &PgPool, preset_id: i64) -> Result<Vec<PabRow>> {
        let rows = sqlx::query_as::<_, PabRow>(
            "SELECT id, preset_id, url FROM scrapper_preset_pabs WHERE preset_id = $1 ORDER BY id",
        )
        .bind(preset_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
