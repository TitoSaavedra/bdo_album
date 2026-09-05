use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(Serialize, sqlx::FromRow)]
pub struct ModificationRow {
    pub modification_id: String,
    pub preset_id:       String,
    pub image_1_url:     String,
    pub image_2_url:     Option<String>,
    pub pab_url:         Option<String>,
    pub created_at:      i64,
}

pub struct ModificationRepository;

impl ModificationRepository {
    pub async fn list_by_preset(
        pool:          &PgPool,
        preset_id:     i64,
        r2_public_url: &str,
    ) -> Result<Vec<ModificationRow>> {
        let rows = sqlx::query_as::<_, ModificationRow>(
            r#"
            SELECT
                id::TEXT                                                    AS modification_id,
                preset_id::TEXT                                             AS preset_id,
                $2 || image_1_url                                           AS image_1_url,
                CASE WHEN image_2_url IS NOT NULL THEN $2 || image_2_url END AS image_2_url,
                CASE WHEN pab_url IS NOT NULL THEN $2 || pab_url END        AS pab_url,
                EXTRACT(EPOCH FROM created_at)::BIGINT                      AS created_at
            FROM album_preset_modifications
            WHERE preset_id = $1
            ORDER BY id
            "#,
        )
        .bind(preset_id)
        .bind(r2_public_url)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

pub struct PendingModificationRepository;

impl PendingModificationRepository {
    /// Stages raw pasted image bytes (plus, optionally, the .pab file
    /// currently sitting in the local BDO export folder — see
    /// `beauty::commands::upload_preset_modification`) — album never talks
    /// to R2 directly for this feature; the scraper's `preset_modifications`
    /// worker drains this queue every 60s, uploads to R2, and purges the row
    /// (see the migration and `scraper/core/src/scraper/preset_modifications.rs`).
    pub async fn insert(
        pool:          &PgPool,
        preset_id:     i64,
        image_1_bytes: &[u8],
        image_2_bytes: Option<&[u8]>,
        pab_bytes:     Option<&[u8]>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_pending_preset_modifications (preset_id, image_1_data, image_2_data, pab_data)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(preset_id)
        .bind(image_1_bytes)
        .bind(image_2_bytes)
        .bind(pab_bytes)
        .execute(pool)
        .await?;
        Ok(())
    }
}
