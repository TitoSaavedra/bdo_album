//! Long-running worker that drains the album's auto-download queue
//! (presets marked "wanted", waiting on a `.pab` file) via the imported
//! Garmoth session. Meant to run as a continuous systemd service (see
//! `../../systemd/bdo-scraper-download-daemon.service`), separate from the
//! oneshot `scrape` binary/timer.
//!
//! Requires `garmoth_auth.json` (an authenticated Discord-OAuth session
//! cookie, exported from the desktop GUI) to be present — see
//! `../../systemd/README.md` for how that gets onto the server. There is no
//! way to obtain this session without a human logging in through a real
//! browser, so that manual step isn't eliminated by this daemon, only
//! relocated from "run the GUI" to "copy one file to the server".

use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;

use bdo_scraper_core::db::pool;
use bdo_scraper_core::events::Sink;
use bdo_scraper_core::scraper::auto_download;

use bdo_scraper_cli::cli_sink::CliSink;

#[derive(Parser, Debug)]
#[command(
    name = "download_daemon",
    about = "Long-running worker that drains the album's auto-download (.pab) queue."
)]
struct Args {
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

    let sink: Arc<dyn Sink> = CliSink::new();
    tracing::info!("download_daemon started — polling auto_download queue every 60s");

    // Never returns under normal operation — systemd (Type=simple,
    // Restart=on-failure) supervises the process instead.
    auto_download::run_loop(sink, pool).await;

    ExitCode::SUCCESS
}
