use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(sqlx::FromRow)]
pub struct SessionRow {
    pub id:             i64,
    pub started_at:     chrono::DateTime<chrono::Utc>,
    pub finished_at:    Option<chrono::DateTime<chrono::Utc>>,
    pub status:         String,
    pub total_fetched:  i32,
    pub total_updated:  i32,
    pub total_images:   i32,
    pub total_uploaded: i32,
    pub errors:         i32,
    pub skipped:        i32,
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

#[derive(sqlx::FromRow)]
pub struct SessionTotals {
    pub count:          i64,
    pub total_fetched:  i64,
    pub total_updated:  i64,
    pub total_images:   i64,
    pub total_uploaded: i64,
    pub errors:         i64,
    pub skipped:        i64,
}

pub struct SessionRepository;

impl SessionRepository {
    pub async fn recover_interrupted(pool: &PgPool) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE scraper_sessions
             SET status = 'interrupted', finished_at = NOW()
             WHERE status = 'running'",
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn cancel_stale(pool: &PgPool) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE scraper_sessions
             SET status = 'cancelled', finished_at = NOW()
             WHERE status = 'running'",
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn create(pool: &PgPool, cf_used: bool) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scraper_sessions (cf_used) VALUES ($1) RETURNING id",
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
        total_updated: i32,
        total_images: i32,
        total_uploaded: i32,
        errors: i32,
        skipped: i32,
        elapsed_secs: i32,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE scraper_sessions SET
                finished_at    = NOW(),
                status         = $1,
                total_fetched  = $2,
                total_updated  = $3,
                total_images   = $4,
                total_uploaded = $5,
                errors         = $6,
                skipped        = $7,
                elapsed_secs   = $8
             WHERE id = $9",
        )
        .bind(status)
        .bind(total_fetched)
        .bind(total_updated)
        .bind(total_images)
        .bind(total_uploaded)
        .bind(errors)
        .bind(skipped)
        .bind(elapsed_secs)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<SessionRow>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, started_at, finished_at, status,
                    total_fetched, total_updated, total_images, total_uploaded,
                    errors, skipped, elapsed_secs, cf_used
             FROM scraper_sessions
             ORDER BY started_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_totals(pool: &PgPool) -> Result<SessionTotals> {
        let row = sqlx::query_as::<_, SessionTotals>(
            "SELECT
                COUNT(*)::bigint                          AS count,
                COALESCE(SUM(total_fetched),  0)::bigint AS total_fetched,
                COALESCE(SUM(total_updated),  0)::bigint AS total_updated,
                COALESCE(SUM(total_images),   0)::bigint AS total_images,
                COALESCE(SUM(total_uploaded), 0)::bigint AS total_uploaded,
                COALESCE(SUM(errors),         0)::bigint AS errors,
                COALESCE(SUM(skipped),        0)::bigint AS skipped
             FROM scraper_sessions",
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn get_class_stats(pool: &PgPool, session_id: i64) -> Result<Vec<ClassStatRow>> {
        let rows = sqlx::query_as::<_, ClassStatRow>(
            "SELECT class_id, fetched, images_ok, errors
             FROM scraper_class_stats
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
            "INSERT INTO scraper_class_stats (session_id, class_id, fetched, images_ok, errors)
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
