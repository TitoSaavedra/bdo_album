use sqlx::PgPool;

use crate::errors::Result;

pub struct PendingPabRow {
    pub id:       i64,
    pub filename: String,
    pub pab_data: Vec<u8>,
}

pub struct PendingPabUploadRepository;

impl PendingPabUploadRepository {
    /// Oldest not-yet-failed staged PAB (album writes these, never touches R2
    /// itself — see the migration's comment on `album_pending_pab_uploads`).
    pub async fn next_pending(pool: &PgPool) -> Result<Option<PendingPabRow>> {
        let row = sqlx::query_as::<_, (i64, String, Vec<u8>)>(
            r#"
            SELECT id, filename, pab_data
            FROM album_pending_pab_uploads
            WHERE error IS NULL
            ORDER BY requested_at
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?
        .map(|(id, filename, pab_data)| PendingPabRow {
            id,
            filename,
            pab_data,
        });
        Ok(row)
    }

    /// Purges a row once its PAB has been uploaded to R2 and recorded in
    /// `scraper_preset_pabs` (or skipped because the preset already has one).
    pub async fn delete(pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM album_pending_pab_uploads WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Marks a row as failed (no preset ID in the filename, unknown preset, R2
    /// hiccup) so it's excluded from future polling instead of retrying forever —
    /// same dead-letter pattern as `PendingModificationRepository::mark_failed`.
    pub async fn mark_failed(pool: &PgPool, id: i64, error: &str) -> Result<()> {
        sqlx::query("UPDATE album_pending_pab_uploads SET error = $2 WHERE id = $1")
            .bind(id)
            .bind(error)
            .execute(pool)
            .await?;
        Ok(())
    }
}
