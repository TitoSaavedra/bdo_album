use std::path::{Path, PathBuf};

use playwright_rs::{Cookie, StorageState};
use serde::Deserialize;

use crate::errors::{AppError, Result};

const SESSION_FILE: &str = "garmoth_auth.json";

/// One entry from a Cookie-Editor ("Export as JSON") dump — matches the shape
/// of `chrome.cookies.Cookie`, which is what Cookie-Editor exports verbatim.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CookieEditorEntry {
    domain: String,
    name: String,
    value: String,
    #[serde(default = "default_path")]
    path: String,
    #[serde(default)]
    expiration_date: Option<f64>,
    #[serde(default)]
    session: bool,
    #[serde(default)]
    http_only: bool,
    #[serde(default)]
    secure: bool,
    #[serde(default)]
    same_site: Option<String>,
}

fn default_path() -> String {
    "/".to_string()
}

/// Maps Chrome's `SameSiteStatus` values (also used by Cookie-Editor on Firefox)
/// to what Playwright expects. Unknown/unspecified values are omitted rather
/// than guessed, matching Playwright's own "no attribute" behavior.
fn map_same_site(value: &str) -> Option<String> {
    match value {
        "strict" => Some("Strict".to_string()),
        "lax" => Some("Lax".to_string()),
        "no_restriction" => Some("None".to_string()),
        _ => None,
    }
}

/// Parses a Cookie-Editor JSON export into a Playwright `StorageState`.
/// Cookie-Editor exports cookies only (no localStorage), which is sufficient
/// for garmoth.com — its session lives entirely in a cookie set after Discord OAuth.
pub fn convert_cookie_editor_export(json: &str) -> Result<StorageState> {
    let entries: Vec<CookieEditorEntry> = serde_json::from_str(json)
        .map_err(|e| AppError::Scrape(format!("invalid cookie export: {e}")))?;

    if entries.is_empty() {
        return Err(AppError::Scrape("cookie export is empty".into()));
    }

    let cookies: Vec<Cookie> = entries
        .into_iter()
        .map(|e| {
            let mut cookie = Cookie::new(e.name, e.value)
                .domain(e.domain)
                .path(e.path)
                .expires(if e.session { -1.0 } else { e.expiration_date.unwrap_or(-1.0) })
                .http_only(e.http_only)
                .secure(e.secure);
            if let Some(ss) = e.same_site.as_deref().and_then(map_same_site) {
                cookie = cookie.same_site(ss);
            }
            cookie
        })
        .collect();

    Ok(StorageState::default().cookies(cookies))
}

/// Resolves where the imported Garmoth session (cookies) lives on disk.
///
/// Resolution order:
/// 1. `GARMOTH_SESSION_FILE` env var, if set — an explicit override, used by
///    the headless CLI (`download_daemon` needs this for `download_pab`; the
///    GUI can also set it, though it normally relies on (2) instead).
/// 2. `base_dir.join("garmoth_auth.json")`, if `base_dir` is given — this is
///    how the GUI calls in today (passing its Tauri `app_data_dir()`,
///    resolved in `src-tauri` before calling down into this function).
/// 3. A cross-platform fallback under the OS data dir (`dirs::data_dir()`) —
///    used when neither of the above is available (e.g. the CLI running
///    without `GARMOTH_SESSION_FILE` set). The default `scrape` binary never
///    needs this path at all: Garmoth's `search-advanced` endpoint is public.
pub fn session_path(base_dir: Option<&Path>) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("GARMOTH_SESSION_FILE") {
        return Ok(PathBuf::from(p));
    }
    if let Some(dir) = base_dir {
        return Ok(dir.join(SESSION_FILE));
    }
    let base = dirs::data_dir().ok_or_else(|| {
        AppError::Scrape(
            "could not resolve a base directory for the Garmoth session file \
             (set GARMOTH_SESSION_FILE)"
                .into(),
        )
    })?;
    Ok(base.join("bdo-scraper").join(SESSION_FILE))
}

pub fn save(base_dir: Option<&Path>, state: &StorageState) -> Result<()> {
    let path = session_path(base_dir)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Scrape(format!("create app data dir: {e}")))?;
    }
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| AppError::Scrape(format!("serialize session: {e}")))?;
    std::fs::write(&path, bytes)
        .map_err(|e| AppError::Scrape(format!("write {}: {e}", path.display())))?;
    Ok(())
}
