use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use sqlx::postgres::PgListener;
use sqlx::PgPool;

use crate::db::repositories::log_repo::LogRepository;
use crate::events::Sink;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Drives a "wake on `NOTIFY <channel>`, otherwise poll every
/// `fallback_interval`" loop — shared by `auto_download` and
/// `preset_modifications` so a queued item is drained as soon as the album
/// queues it instead of waiting out a fixed poll interval. The fallback poll
/// still runs so nothing is lost if the LISTEN connection drops or a NOTIFY
/// fires before this process starts listening (e.g. right after a restart).
pub async fn run<F, Fut>(
    sink: &Arc<dyn Sink>,
    pool: &PgPool,
    channel: &str,
    source: &str,
    fallback_interval: Duration,
    mut on_wake: F,
) where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    loop {
        let mut listener = match PgListener::connect_with(pool).await {
            Ok(l) => l,
            Err(e) => {
                LogRepository::insert(sink.as_ref(), pool, None, "ERR", source,
                    &format!("LISTEN connect failed: {e}")).await.ok();
                tokio::time::sleep(RECONNECT_DELAY).await;
                continue;
            }
        };
        if let Err(e) = listener.listen(channel).await {
            LogRepository::insert(sink.as_ref(), pool, None, "ERR", source,
                &format!("LISTEN {channel} failed: {e}")).await.ok();
            tokio::time::sleep(RECONNECT_DELAY).await;
            continue;
        }

        loop {
            tokio::select! {
                notif = listener.recv() => {
                    if notif.is_err() {
                        break; // connection dropped — reconnect
                    }
                    on_wake().await;
                }
                _ = tokio::time::sleep(fallback_interval) => {
                    on_wake().await;
                }
            }
        }
    }
}
