use sqlx::PgPool;

use crate::core::errors::Result;
use crate::db::repositories::{
    class_repo::{ClassRepository, ClassRow},
    preset_repo::{PresetRepository, PresetRow},
};

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
        r2_public_url: &str,
    ) -> Result<Vec<PresetRow>> {
        let class_id = PresetRepository::get_class_id(pool, class_name).await?;
        let Some(class_id) = class_id else {
            return Ok(vec![]);
        };
        PresetRepository::get_by_class(pool, class_id, offset, limit, sort_by, search, r2_public_url).await
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
}
