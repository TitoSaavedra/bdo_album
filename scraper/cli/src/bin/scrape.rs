//! Headless one-shot BDO Beauty Album scrape (Garmoth), meant to run under a
//! systemd timer (see `../../systemd/bdo-scraper.{service,timer}`).
//!
//! Unlike the desktop GUI's "scrape everything" shortcut (an empty `classes`
//! filter, which the GUI maps to Garmoth's single global-ranking endpoint —
//! see `src-tauri/src/scraper/commands.rs::run_scraper`), this binary's
//! default with no `--classes` flag is to walk every seeded class
//! individually. That's the whole point of running this unattended: a full
//! sweep, not a shortcut.

use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::Parser;

use bdo_scraper_core::db::pool;
use bdo_scraper_core::db::repositories::{
    class_repo::ClassRepository, log_repo::LogRepository, session_repo::SessionRepository,
};
use bdo_scraper_core::events::{LogCode, Sink};
use bdo_scraper_core::scraper::defaults::{resolve_days, resolve_regions};
use bdo_scraper_core::scraper::service;
use bdo_scraper_core::session_guard::SessionGuard;

use bdo_scraper_cli::cli_sink::CliSink;

#[derive(Parser, Debug)]
#[command(
    name = "scrape",
    about = "Headless one-shot BDO Beauty Album scrape (Garmoth). Intended to run under a systemd timer."
)]
struct Args {
    /// Comma-separated class ids (e.g. `1,2,3`). If omitted, every seeded
    /// class is scraped individually — NOT the GUI's "all" global-ranking
    /// shortcut.
    #[arg(long, value_delimiter = ',')]
    classes: Vec<i32>,

    /// Comma-separated day windows (e.g. `20,30,60`). Defaults to the same
    /// 7-value set the GUI uses.
    #[arg(long, value_delimiter = ',')]
    days: Vec<String>,

    /// Comma-separated regions (e.g. `eu,na,kr`). Defaults to the same
    /// 10-value set the GUI uses.
    #[arg(long, value_delimiter = ',')]
    regions: Vec<String>,

    /// Concurrent in-flight requests / downloads.
    #[arg(long, default_value_t = 2)]
    parallelism: usize,

    /// fetch | images | both
    #[arg(long, default_value = "both")]
    mode: String,

    /// Postgres connection string. Falls back to `DATABASE_URL` from the
    /// exe-adjacent `.env`, then the process environment, when omitted.
    #[arg(long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dotenvy::from_path(dir.join(".env")).ok();
        }
    }
    dotenvy::dotenv().ok();

    let args = Args::parse();

    let database_url = match args
        .database_url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok())
    {
        Some(url) => url,
        None => {
            tracing::error!("DATABASE_URL not set (checked --database-url, .env, environment)");
            return ExitCode::FAILURE;
        }
    };

    let pool = match pool::init(&database_url).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("failed to connect to database: {e}");
            return ExitCode::FAILURE;
        }
    };

    // The real cross-process authority: a Postgres advisory lock, so this CLI
    // and the desktop GUI (or two overlapping timer runs) can never scrape
    // the same production DB concurrently. See `session_guard` docs.
    let guard = match SessionGuard::try_acquire(&database_url).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            tracing::warn!("another scraper session is already running (advisory lock held) — exiting");
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            tracing::error!("failed to acquire session lock: {e}");
            return ExitCode::FAILURE;
        }
    };

    let sink: Arc<dyn Sink> = CliSink::new();

    let parallelism = args.parallelism.max(1);
    let mode = args.mode.clone();
    let days = resolve_days(args.days);
    let regions = resolve_regions(args.regions);

    let classes: Vec<serde_json::Value> = if args.classes.is_empty() {
        match ClassRepository::get_all(&pool).await {
            Ok(rows) => rows.into_iter().map(|r| serde_json::json!(r.id)).collect(),
            Err(e) => {
                tracing::error!("failed to load classes: {e}");
                guard.release().await;
                return ExitCode::FAILURE;
            }
        }
    } else {
        args.classes.iter().map(|id| serde_json::json!(id)).collect()
    };

    let cancel = Arc::new(AtomicBool::new(false));
    setup_signal_handler(Arc::clone(&cancel));

    SessionRepository::cancel_stale(&pool).await.ok();
    let session_id = match SessionRepository::create(&pool, true).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("failed to create session row: {e}");
            guard.release().await;
            return ExitCode::FAILURE;
        }
    };

    sink.scraper_started();

    let classes_str: Vec<String> = classes
        .iter()
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_i64() {
                n.to_string()
            } else {
                "?".to_string()
            }
        })
        .collect();

    LogRepository::insert_coded(
        sink.as_ref(),
        &pool,
        Some(session_id),
        "ORCH",
        "session",
        &format!(
            "Session #{} started — mode={} | parallelism={} | classes=[{}] | days=[{}] | regions=[{}]",
            session_id,
            mode,
            parallelism,
            classes_str.join(", "),
            days.join(", "),
            regions.join(", "),
        ),
        Some(LogCode::SessionStarted {
            session_id,
            mode: mode.clone(),
            parallelism: parallelism as i64,
            classes: classes_str.join(", "),
            days: days.join(", "),
            regions: regions.join(", "),
        }),
    )
    .await
    .ok();

    // Runs to completion (or cancellation) and releases `guard` itself —
    // see `service::abort_session`/`finish_session`.
    service::run_session(
        sink, pool, cancel, session_id, parallelism, days, regions, classes, mode, guard, None,
        None,
    )
    .await;

    ExitCode::SUCCESS
}

/// systemd sends SIGTERM on `stop` and when the timer preempts a still-running
/// unit — wired into the same `cancel: Arc<AtomicBool>` flag `run_session`
/// already checks everywhere, so this reuses the existing cancellation path
/// unchanged.
#[cfg(unix)]
fn setup_signal_handler(cancel: Arc<AtomicBool>) {
    use std::sync::atomic::Ordering;

    tokio::spawn(async move {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
            tracing::warn!("received SIGTERM — cancelling session");
            cancel.store(true, Ordering::Relaxed);
        }
    });
}

/// This binary's real target is Linux/systemd, where SIGTERM handling above
/// applies. `tokio::signal::unix` doesn't exist on other targets — this
/// no-op fallback exists only so the crate still typechecks on non-unix dev
/// machines (e.g. `cargo check --workspace` on Windows).
#[cfg(not(unix))]
fn setup_signal_handler(_cancel: Arc<AtomicBool>) {}
