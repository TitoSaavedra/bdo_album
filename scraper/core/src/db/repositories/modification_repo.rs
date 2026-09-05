use sqlx::PgPool;

use crate::errors::Result;

pub struct PendingRow {
    pub id:            i64,
    pub preset_id:      i64,
    pub image_1_data:   Vec<u8>,
    pub image_2_data:   Option<Vec<u8>>,
}

pub struct PendingModificationRepository;

impl PendingModificationRepository {
    /// Oldest not-yet-failed staged row (album writes these, never touches R2
    /// itself — see the migration's comment on `album_pending_preset_modifications`).
    pub async fn next_pending(pool: &PgPool) -> Result<Option<PendingRow>> {
        let row = sqlx::query_as::<_, (i64, i64, Vec<u8>, Option<Vec<u8>>)>(
            r#"
            SELECT id, preset_id, image_1_data, image_2_data
            FROM album_pending_preset_modifications
            WHERE error IS NULL
            ORDER BY requested_at
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?
        .map(|(id, preset_id, image_1_data, image_2_data)| PendingRow {
            id,
            preset_id,
            image_1_data,
            image_2_data,
        });
        Ok(row)
    }

    /// Purges a row once its images have been uploaded to R2 and the finished
    /// row is inserted into `album_preset_modifications`.
    pub async fn delete(pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM album_pending_preset_modifications WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Marks a row as failed (bad image data, R2 hiccup) so it's excluded from
    /// future polling instead of retrying forever — same dead-letter pattern
    /// as `album_user_prefs.auto_download_error`.
    pub async fn mark_failed(pool: &PgPool, id: i64, error: &str) -> Result<()> {
        sqlx::query("UPDATE album_pending_preset_modifications SET error = $2 WHERE id = $1")
            .bind(id)
            .bind(error)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct ModificationRepository;

impl ModificationRepository {
    /// Inserts the finished, R2-backed row. `image_1_key`/`image_2_key` are R2
    /// keys (not full URLs) — same convention as `scraper_presets.image_1_url`.
    pub async fn insert(
        pool:        &PgPool,
        preset_id:   i64,
        image_1_key: &str,
        image_2_key: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_preset_modifications (preset_id, image_1_url, image_2_url)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(preset_id)
        .bind(image_1_key)
        .bind(image_2_key)
        .execute(pool)
        .await?;
        Ok(())
    }
}
