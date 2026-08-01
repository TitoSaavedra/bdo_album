use std::sync::Arc;

use serde::Deserialize;

use crate::core::errors::{AppError, Result};

use super::browser::BrowserSession;

const API_BASE: &str = "https://api.garmoth.com";

pub struct GarmothPreset {
    pub id:             i64,
    pub class_id:       i32,
    pub title:          Option<String>,
    pub user_nickname:  Option<String>,
    pub character_name: Option<String>,
    pub downloads:      i64,
    pub views:          i64,
    pub likes:          i64,
    pub image_1:        Option<String>,
    pub image_2:        Option<String>,
    pub creation_at:    Option<i64>,
    pub customizing_id: Option<i64>,
    pub region:         Option<String>,
    pub score:          Option<i64>,
}

#[derive(Deserialize)]
struct RawPreset {
    id:             i64,
    class:          i32,
    title:          Option<String>,
    user_nickname:  Option<String>,
    character_name: Option<String>,
    #[serde(default)]
    downloads:      i64,
    #[serde(default)]
    views:          i64,
    #[serde(default)]
    likes:          i64,
    image_1:        Option<String>,
    image_2:        Option<String>,
    #[serde(rename = "created_at")]
    creation_at:    Option<i64>,
    customizing_id: Option<i64>,
    region:         Option<String>,
    score:          Option<i64>,
}

impl From<RawPreset> for GarmothPreset {
    fn from(r: RawPreset) -> Self {
        Self {
            id:             r.id,
            class_id:       r.class,
            title:          r.title,
            user_nickname:  r.user_nickname,
            character_name: r.character_name,
            downloads:      r.downloads,
            views:          r.views,
            likes:          r.likes,
            image_1:        r.image_1,
            image_2:        r.image_2,
            creation_at:    r.creation_at,
            customizing_id: r.customizing_id,
            region:         r.region,
            score:          r.score,
        }
    }
}

// Queries go through the browser's own Chromium network stack (see
// BrowserSession::fetch_json) rather than a standalone reqwest client — Cloudflare's
// WAF fingerprints the TLS handshake itself and blocks non-browser clients with a 403
// regardless of what headers/cookies are attached.
pub struct GarmothClient {
    browser: Arc<BrowserSession>,
}

impl GarmothClient {
    pub fn new(browser: Arc<BrowserSession>) -> Self {
        Self { browser }
    }

    pub async fn fetch_popular(
        &self,
        class_id: Option<u32>,
        days: &str,
        region: &str,
    ) -> Result<Vec<GarmothPreset>> {
        let class_param = class_id.map_or_else(|| "all".to_string(), |id| id.to_string());
        let url = format!(
            "{}/api/beauty-album/search-advanced?class={}&past={}&region={}&sort=popular&limit=100",
            API_BASE, class_param, days, region
        );

        let bytes = self.browser.fetch_json(&url).await?;

        let val: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Scrape(format!("parse failed: {}", e)))?;

        let arr = val["presets"]["data"].as_array().cloned().unwrap_or_default();
        let presets = arr
            .into_iter()
            .filter_map(|v| serde_json::from_value::<RawPreset>(v).ok())
            .map(GarmothPreset::from)
            .collect();
        Ok(presets)
    }
}
