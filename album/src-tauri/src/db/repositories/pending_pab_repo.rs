use sqlx::PgPool;

use crate::core::errors::Result;

pub struct PendingPabUploadRepository;

impl PendingPabUploadRepository {
    /// Stages a dragged-in .pab file (raw bytes + original filename) for the
    /// scraper's `pab_uploads` worker, which does all validation (preset id
    /// from the filename, preset exists, no PAB yet), uploads to R2 and purges
    /// the row (see the migration `0019_album_pending_pab_uploads.sql`).
    /// Album never talks to R2 for this feature.
    pub async fn insert(pool: &PgPool, filename: &str, pab_bytes: &[u8]) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_pending_pab_uploads (filename, pab_data)
            VALUES ($1, $2)
            "#,
        )
        .bind(filename)
        .bind(pab_bytes)
        .execute(pool)
        .await?;

        sqlx::query("SELECT pg_notify('pab_upload_queued', $1)")
            .bind(serde_json::json!({ "filename": filename }).to_string())
            .execute(pool)
            .await?;
        Ok(())
    }
}
