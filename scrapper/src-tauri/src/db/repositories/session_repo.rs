use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(sqlx::FromRow)]
pub struct SessionRow {
    pub id:             i64,
    pub started_at:     chrono::DateTime<chrono::Utc>,
    pub finished_at:    Option<chrono::DateTime<chrono::Utc>>,
    pub status:         String,
    pub total_fetched:  i32,
    pub total_images:   i32,
    pub total_uploaded: i32,
    pub errors:         i32,
    pub elapsed_secs:   Option<i32>,
    pub cf_used:        bool,
}

#[derive(sqlx::FromRow)]
pub struct ClassStatRow {
    pub class_id:  i32,
    pub fetched:   i32,
    pub images_ok: i32,
    pub errors:    i32,
}

pub struct SessionRepository;

impl SessionRepository {
    pub async fn create(pool: &PgPool, cf_used: bool) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scrapper_sessions (cf_used) VALUES ($1) RETURNING id",
        )
        .bind(cf_used)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }

    pub async fn finish(
        pool: &PgPool,
        id: i64,
        status: &str,
        total_fetched: i32,
        total_images: i32,
        total_uploaded: i32,
        errors: i32,
        elapsed_secs: i32,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE scrapper_sessions SET
                finished_at    = NOW(),
                status         = $1,
                total_fetched  = $2,
                total_images   = $3,
                total_uploaded = $4,
                errors         = $5,
                elapsed_secs   = $6
             WHERE id = $7",
        )
        .bind(status)
        .bind(total_fetched)
        .bind(total_images)
        .bind(total_uploaded)
        .bind(errors)
        .bind(elapsed_secs)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<SessionRow>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, started_at, finished_at, status,
                    total_fetched, total_images, total_uploaded,
                    errors, elapsed_secs, cf_used
             FROM scrapper_sessions
             ORDER BY started_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_class_stats(pool: &PgPool, session_id: i64) -> Result<Vec<ClassStatRow>> {
        let rows = sqlx::query_as::<_, ClassStatRow>(
            "SELECT class_id, fetched, images_ok, errors
             FROM scrapper_class_stats
             WHERE session_id = $1
             ORDER BY fetched DESC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn upsert_class_stats(
        pool: &PgPool,
        session_id: i64,
        class_id: i32,
        fetched: i32,
        images_ok: i32,
        errors: i32,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO scrapper_class_stats (session_id, class_id, fetched, images_ok, errors)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (session_id, class_id) DO UPDATE SET
                fetched     = EXCLUDED.fetched,
                images_ok   = EXCLUDED.images_ok,
                errors      = EXCLUDED.errors,
                snapshot_at = NOW()",
        )
        .bind(session_id)
        .bind(class_id)
        .bind(fetched)
        .bind(images_ok)
        .bind(errors)
        .execute(pool)
        .await?;
        Ok(())
    }
}
