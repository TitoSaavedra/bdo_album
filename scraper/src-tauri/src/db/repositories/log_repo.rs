use sqlx::PgPool;
use tauri::AppHandle;

use crate::core::errors::Result;
use crate::events::{Events, LogCode, LogEntry};

#[derive(sqlx::FromRow)]
pub struct LogRow {
    pub id:         i64,
    pub ts:         chrono::DateTime<chrono::Utc>,
    pub session_id: Option<i64>,
    pub tag:        String,
    pub source:     String,
    pub msg:        String,
}

pub struct LogRepository;

impl LogRepository {
    pub async fn insert(
        app:        &AppHandle,
        pool:       &PgPool,
        session_id: Option<i64>,
        tag:        &str,
        source:     &str,
        msg:        &str,
    ) -> Result<()> {
        Self::insert_coded(app, pool, session_id, tag, source, msg, None).await
    }

    /// Same as [`insert`], but also attaches a [`LogCode`] to the live Tauri event
    /// so the frontend can render a localized sentence. `msg` (English, built by the
    /// caller) is always what's persisted to `scraper_logs` — `code` is never stored.
    pub async fn insert_coded(
        app:        &AppHandle,
        pool:       &PgPool,
        session_id: Option<i64>,
        tag:        &str,
        source:     &str,
        msg:        &str,
        code:       Option<LogCode>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO scraper_logs (session_id, tag, source, msg) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(tag)
        .bind(source)
        .bind(msg)
        .execute(pool)
        .await?;

        Events::log_entry(app, LogEntry {
            ts:     chrono::Utc::now().timestamp(),
            tag:    tag.to_string(),
            source: source.to_string(),
            msg:    msg.to_string(),
            code,
        });

        Ok(())
    }

    /// Deletes log rows older than `days` days. Called once at startup to keep
    /// scraper_logs from growing unbounded over months of daily use.
    pub async fn prune(pool: &PgPool, days: i64) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM scraper_logs WHERE ts < now() - ($1 || ' days')::interval",
        )
        .bind(days)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<LogRow>> {
        let rows = sqlx::query_as::<_, LogRow>(
            "SELECT id, ts, session_id, tag, source, msg
             FROM scraper_logs
             ORDER BY ts DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
