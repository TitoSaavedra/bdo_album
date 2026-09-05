use sqlx::PgPool;

use crate::errors::Result;

pub struct PendingRow {
    pub id:          i64,
    pub preset_id:   i64,
    pub class_id:    i32,
    pub image_1_data: Vec<u8>,
    pub image_2_data: Option<Vec<u8>>,
    pub pab_data:    Option<Vec<u8>>,
}

pub struct PendingModificationRepository;

impl PendingModificationRepository {
    /// Oldest not-yet-failed staged row (album writes these, never touches R2
    /// itself — see the migration's comment on `album_pending_preset_modifications`).
    /// Joins `scraper_presets` for `class_id`, needed for the `preset_uploaded`
    /// live-notify payload once the upload succeeds.
    pub async fn next_pending(pool: &PgPool) -> Result<Option<PendingRow>> {
        let row = sqlx::query_as::<_, (i64, i64, i32, Vec<u8>, Option<Vec<u8>>, Option<Vec<u8>>)>(
            r#"
            SELECT m.id, m.preset_id, p.class_id, m.image_1_data, m.image_2_data, m.pab_data
            FROM album_pending_preset_modifications m
            JOIN scraper_presets p ON p.id = m.preset_id
            WHERE m.error IS NULL
            ORDER BY m.requested_at
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?
        .map(|(id, preset_id, class_id, image_1_data, image_2_data, pab_data)| PendingRow {
            id,
            preset_id,
            class_id,
            image_1_data,
            image_2_data,
            pab_data,
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
    /// How many finished modifications this preset already has — used to name
    /// the next one `modificacion_{n+1}` (1-indexed, per preset).
    pub async fn count_by_preset(pool: &PgPool, preset_id: i64) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM album_preset_modifications WHERE preset_id = $1",
        )
        .bind(preset_id)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    /// Inserts the finished, R2-backed row. `image_1_key`/`image_2_key`/`pab_key`
    /// are R2 keys (not full URLs) — same convention as `scraper_presets.image_1_url`.
    pub async fn insert(
        pool:        &PgPool,
        preset_id:   i64,
        image_1_key: &str,
        image_2_key: Option<&str>,
        pab_key:     Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_preset_modifications (preset_id, image_1_url, image_2_url, pab_url)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(preset_id)
        .bind(image_1_key)
        .bind(image_2_key)
        .bind(pab_key)
        .execute(pool)
        .await?;
        Ok(())
    }
}
