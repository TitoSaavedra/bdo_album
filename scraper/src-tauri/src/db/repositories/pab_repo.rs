use sqlx::PgPool;

use crate::core::errors::Result;

pub struct PabRepository;

#[derive(sqlx::FromRow)]
pub struct PabRow {
    pub url: String,
}

#[derive(sqlx::FromRow)]
pub struct PabSearchRow {
    pub preset_id:      i64,
    pub character_name: Option<String>,
    pub class_name:     String,
    pub url:            String,
    pub synced_at:      chrono::DateTime<chrono::Utc>,
}

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

    /// Most recently synced PAB row for a preset, if one exists.
    pub async fn find_by_preset(pool: &PgPool, preset_id: i64) -> Result<Option<PabRow>> {
        let row = sqlx::query_as::<_, PabRow>(
            "SELECT url FROM scraper_preset_pabs
             WHERE preset_id = $1
             ORDER BY synced_at DESC
             LIMIT 1",
        )
        .bind(preset_id)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    /// Updates the URL of an existing PAB row for `preset_id`, or inserts one if
    /// none exists yet. Used by the repair flow to point at the freshly re-uploaded
    /// (unpatched) file without leaving a duplicate row behind.
    pub async fn replace_url(pool: &PgPool, preset_id: i64, url: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE scraper_preset_pabs SET url = $1, synced_at = NOW() WHERE preset_id = $2",
        )
        .bind(url)
        .bind(preset_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            Self::insert(pool, preset_id, url).await?;
        }
        Ok(())
    }

    /// Presets that already have a PAB uploaded, matched by preset ID, character
    /// name, or class — feeds the "repair a broken PAB" search in the dashboard.
    pub async fn search(pool: &PgPool, query: &str) -> Result<Vec<PabSearchRow>> {
        let like = format!("%{}%", query.trim());
        let rows = sqlx::query_as::<_, PabSearchRow>(
            "SELECT p.id AS preset_id, p.character_name, c.display AS class_name,
                    pab.url, pab.synced_at
             FROM scraper_preset_pabs pab
             JOIN scraper_presets p ON p.id = pab.preset_id
             JOIN scraper_classes c ON c.id = p.class_id
             WHERE $1 = ''
                OR p.id::text ILIKE $2
                OR p.character_name ILIKE $2
                OR c.display ILIKE $2
             ORDER BY pab.synced_at DESC
             LIMIT 50",
        )
        .bind(query.trim())
        .bind(&like)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
