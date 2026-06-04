use sqlx::PgPool;
use tauri::AppHandle;

use crate::core::errors::Result;
use crate::events::{Events, LogEntry};

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
        sqlx::query(
            "INSERT INTO scrapper_logs (session_id, tag, source, msg) VALUES ($1, $2, $3, $4)",
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
        });

        Ok(())
    }

    pub async fn get_recent(pool: &PgPool, limit: i64) -> Result<Vec<LogRow>> {
        let rows = sqlx::query_as::<_, LogRow>(
            "SELECT id, ts, session_id, tag, source, msg
             FROM scrapper_logs
             ORDER BY ts DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
