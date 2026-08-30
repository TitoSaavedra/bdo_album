use sqlx::PgPool;

use crate::core::errors::Result;

pub struct AutoDownloadRepository;

impl AutoDownloadRepository {
    /// Oldest queued preset that hasn't errored yet and doesn't already have a PAB.
    pub async fn next_pending(pool: &PgPool) -> Result<Option<i64>> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT u.preset_id
            FROM album_user_prefs u
            WHERE u.auto_download_requested_at IS NOT NULL
              AND u.auto_download_error IS NULL
              AND NOT EXISTS (
                SELECT 1 FROM scraper_preset_pabs WHERE preset_id = u.preset_id
              )
            ORDER BY u.auto_download_requested_at
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;
        Ok(id)
    }

    pub async fn mark_done(pool: &PgPool, preset_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE album_user_prefs
             SET auto_download_requested_at = NULL, auto_download_error = NULL
             WHERE preset_id = $1",
        )
        .bind(preset_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(pool: &PgPool, preset_id: i64, error: &str) -> Result<()> {
        sqlx::query(
            "UPDATE album_user_prefs SET auto_download_error = $2 WHERE preset_id = $1",
        )
        .bind(preset_id)
        .bind(error)
        .execute(pool)
        .await?;
        Ok(())
    }
}
