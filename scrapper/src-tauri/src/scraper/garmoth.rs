use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::Deserialize;

use crate::core::errors::{AppError, Result};

const API_BASE:   &str = "https://api.garmoth.com";
const IMAGE_BASE: &str = "https://assets.garmoth.com/beauty-album/images";

pub struct GarmothPreset {
    pub id:             i64,
    pub class_id:       i32,
    pub title:          Option<String>,
    pub user_nickname:  Option<String>,
    pub character_name: Option<String>,
    pub downloads:      i64,
    pub views:          i64,
    pub likes:          i64,
    pub image_1:        Option<String>,   // raw filename e.g. "42137_1.png"
    pub image_2:        Option<String>,
    pub image_1_url:    Option<String>,   // full Garmoth CDN URL (for browser download)
    pub image_2_url:    Option<String>,
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
        let image_1_url = r.image_1.as_deref()
            .map(|f| format!("{}/{}/{}", IMAGE_BASE, r.class, f));
        let image_2_url = r.image_2.as_deref()
            .map(|f| format!("{}/{}/{}", IMAGE_BASE, r.class, f));
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
            image_1_url,
            image_2_url,
            creation_at:    r.creation_at,
            customizing_id: r.customizing_id,
            region:         r.region,
            score:          r.score,
        }
    }
}

pub struct GarmothClient {
    client: reqwest::Client,
}

impl GarmothClient {
    pub fn new(cf_clearance: &str) -> Self {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:151.0) Gecko/20100101 Firefox/151.0";
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(header::REFERER, HeaderValue::from_static("https://garmoth.com/"));
        headers.insert(header::ORIGIN, HeaderValue::from_static("https://garmoth.com"));
        headers.insert("lang", HeaderValue::from_static("us"));
        if !cf_clearance.is_empty() {
            let cookie = format!("cf_clearance={}", cf_clearance);
            if let Ok(val) = HeaderValue::from_str(&cookie) {
                headers.insert(header::COOKIE, val);
            }
        }
        let client = reqwest::Client::builder()
            .user_agent(ua)
            .default_headers(headers)
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    pub async fn fetch_popular(
        &self,
        class_id: u32,
        days: &str,
        region: &str,
    ) -> Result<Vec<GarmothPreset>> {
        let url = format!(
            "{}/api/beauty-album/search-advanced?class={}&past={}&region={}&sort=popular&limit=100",
            API_BASE, class_id, days, region
        );
        let bytes = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Scrape(format!("request failed: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::Scrape(format!("HTTP error: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Scrape(format!("read failed: {}", e)))?;

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
