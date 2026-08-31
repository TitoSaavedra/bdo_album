use sqlx::{Connection, PgConnection};

use crate::errors::Result;

// Same key used consistently everywhere this lock is taken/released — Postgres
// advisory locks are keyed by a bigint, `hashtext(...)` gives a stable one from
// a human-readable name without a lock-name registry to keep in sync.
const LOCK_QUERY: &str = "SELECT pg_try_advisory_lock(hashtext('bdo_scraper_session'))";
const UNLOCK_QUERY: &str = "SELECT pg_advisory_unlock(hashtext('bdo_scraper_session'))";

/// Cross-process mutex for "only one scraper session at a time" — the real
/// authority between the desktop GUI and the headless CLI, both of which may
/// point at the same production Postgres. Backed by a Postgres advisory lock
/// held on a *dedicated* (non-pooled) connection, same pattern already used by
/// `scripts/migrate.py` (`pg_advisory_lock(hashtext('bdo_album_migrations'))`).
///
/// The connection is deliberately not drawn from the app's pool: the lock's
/// lifetime is tied 1:1 to this connection's lifetime. If the process crashes
/// before calling [`release`](SessionGuard::release), Postgres releases the
/// advisory lock automatically when the connection drops — the orphaned
/// `scraper_sessions` row that leaves behind is already handled by the
/// existing `SessionRepository::recover_interrupted`/`cancel_stale` sweep.
pub struct SessionGuard {
    conn: Option<PgConnection>,
}

impl SessionGuard {
    /// Attempts to acquire the lock. Returns `Ok(None)` (not an error) when
    /// another process already holds it — that's the expected "scraper
    /// already running" case, not a failure.
    pub async fn try_acquire(database_url: &str) -> Result<Option<Self>> {
        let mut conn = PgConnection::connect(database_url).await?;
        let acquired: bool = sqlx::query_scalar(LOCK_QUERY)
            .fetch_one(&mut conn)
            .await?;

        if acquired {
            Ok(Some(Self { conn: Some(conn) }))
        } else {
            conn.close().await.ok();
            Ok(None)
        }
    }

    /// Releases the advisory lock and closes the dedicated connection. Takes
    /// `self` by value so a session can release it exactly once, at the end
    /// of its lifecycle (mirrors `abort_session`/`finish_session` in
    /// `scraper::service`).
    pub async fn release(mut self) {
        if let Some(mut conn) = self.conn.take() {
            sqlx::query(UNLOCK_QUERY).execute(&mut conn).await.ok();
            conn.close().await.ok();
        }
    }
}
