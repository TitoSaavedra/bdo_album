use sqlx::PgPool;

use crate::core::errors::Result;
use crate::db::repositories::{
    class_repo::{ClassRepository, ClassRow},
    preset_repo::{PresetRepository, PresetRow},
};

fn days_to_cutoff(days: Option<&str>) -> Option<i64> {
    match days {
        None | Some("ever") | Some("") => None,
        Some(d) => d.parse::<i64>().ok().map(|n| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            now - n * 86400
        }),
    }
}

pub struct BeautyService;

impl BeautyService {
    pub async fn get_classes(pool: &PgPool) -> Result<Vec<ClassRow>> {
        ClassRepository::get_all(pool).await
    }

    pub async fn get_class_favorites(pool: &PgPool) -> Result<Vec<String>> {
        ClassRepository::get_favorites(pool).await
    }

    pub async fn set_class_favorite(pool: &PgPool, class_name: &str, is_fav: bool) -> Result<()> {
        ClassRepository::set_favorite(pool, class_name, is_fav).await
    }

    pub async fn get_presets(
        pool:          &PgPool,
        class_name:    &str,
        offset:        i64,
        limit:         i64,
        sort_by:       &str,
        search:        &str,
        region:        Option<&str>,
        days:          Option<&str>,
        r2_public_url: &str,
    ) -> Result<Vec<PresetRow>> {
        let class_id = PresetRepository::get_class_id(pool, class_name).await?;
        let Some(class_id) = class_id else {
            return Ok(vec![]);
        };
        let days_cutoff = days_to_cutoff(days);
        PresetRepository::get_by_class(pool, class_id, offset, limit, sort_by, search, region, days_cutoff, r2_public_url).await
    }

    pub async fn get_preset_by_id(pool: &PgPool, preset_id: i64, r2_public_url: &str) -> Result<Option<PresetRow>> {
        PresetRepository::get_by_id(pool, preset_id, r2_public_url).await
    }

    pub async fn get_regions(pool: &PgPool) -> Result<Vec<String>> {
        PresetRepository::get_distinct_regions(pool).await
    }

    pub async fn discard_preset(pool: &PgPool, preset_id: i64) -> Result<()> {
        PresetRepository::upsert_discard(pool, preset_id).await
    }

    pub async fn toggle_wanted(pool: &PgPool, preset_id: i64) -> Result<bool> {
        PresetRepository::toggle_wanted(pool, preset_id).await
    }

    pub async fn get_wanted_ids(pool: &PgPool) -> Result<Vec<String>> {
        PresetRepository::get_wanted_ids(pool).await
    }

    pub async fn get_wanted_pab_urls(pool: &PgPool) -> Result<Vec<String>> {
        PresetRepository::get_wanted_pab_urls(pool).await
    }

    pub async fn get_wanted_presets(pool: &PgPool, r2_public_url: &str) -> Result<Vec<PresetRow>> {
        PresetRepository::get_wanted_presets(pool, r2_public_url).await
    }

    pub async fn queue_auto_download(pool: &PgPool, preset_ids: &[i64]) -> Result<()> {
        PresetRepository::queue_auto_download(pool, preset_ids).await
    }

    /// Flags a PAB as editable in-game by setting byte `0x44`. The scraper
    /// uploads pristine originals to R2 (see `upload_pab` there for why) — this
    /// patch is applied here instead, right before the file is actually used, so
    /// a bad guess only affects this one export instead of permanently
    /// corrupting the copy of record in R2.
    ///
    /// Offset 0x44 isn't a stable flag position across every class's PAB layout;
    /// when a file is too short to safely carry it, the original bytes are
    /// returned untouched rather than guessing at a different offset.
    pub fn patch_editable(mut bytes: Vec<u8>) -> Vec<u8> {
        if bytes.len() > 0x44 {
            bytes[0x44] = 0xCC;
        }
        bytes
    }

    pub async fn get_class_search_counts(
        pool:   &PgPool,
        search: &str,
        region: Option<&str>,
        days:   Option<&str>,
    ) -> Result<Vec<(i32, i64)>> {
        let days_cutoff = days_to_cutoff(days);
        PresetRepository::count_by_search(pool, search, region, days_cutoff).await
    }
}
