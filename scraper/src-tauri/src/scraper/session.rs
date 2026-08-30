use std::path::PathBuf;

use playwright_rs::{Cookie, StorageState};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::errors::{AppError, Result};

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

pub fn session_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Scrape(format!("resolve app data dir: {e}")))?;
    Ok(dir.join(SESSION_FILE))
}

pub fn save(app: &AppHandle, state: &StorageState) -> Result<()> {
    let path = session_path(app)?;
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
