use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

/// A staged row that still exists — `error` set means the scraper rejected it;
/// rows that are gone were uploaded (or skipped as duplicates) and purged.
#[derive(Serialize, sqlx::FromRow)]
pub struct PendingPabStatus {
    pub id:    i64,
    pub error: Option<String>,
}

pub struct PendingPabUploadRepository;

impl PendingPabUploadRepository {
    /// Stages a dragged-in .pab file (raw bytes + original filename) for the
    /// scraper's `pab_uploads` worker, which does all validation (preset id
    /// from the filename, preset exists, no PAB yet), uploads to R2 and purges
    /// the row (see the migration `0019_album_pending_pab_uploads.sql`).
    /// Album never talks to R2 for this feature. Returns the staged row id.
    pub async fn insert(pool: &PgPool, filename: &str, pab_bytes: &[u8]) -> Result<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO album_pending_pab_uploads (filename, pab_data)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(filename)
        .bind(pab_bytes)
        .fetch_one(pool)
        .await?;

        sqlx::query("SELECT pg_notify('pab_upload_queued', $1)")
            .bind(serde_json::json!({ "filename": filename }).to_string())
            .execute(pool)
            .await?;
        Ok(id)
    }

    /// Rows among `ids` still in the staging table (pending or failed).
    pub async fn status(pool: &PgPool, ids: &[i64]) -> Result<Vec<PendingPabStatus>> {
        let rows = sqlx::query_as::<_, PendingPabStatus>(
            "SELECT id, error FROM album_pending_pab_uploads WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
