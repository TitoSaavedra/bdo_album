use sqlx::PgPool;

use crate::core::errors::Result;

pub struct PresetRepository;

impl PresetRepository {
    /// Inserts a new preset. Returns `true` if inserted, `false` if it already existed.
    pub async fn insert_new(
        pool:           &PgPool,
        id:             i64,
        class_id:       i32,
        title:          Option<&str>,
        user_nickname:  Option<&str>,
        character_name: Option<&str>,
        downloads:      i64,
        views:          i64,
        likes:          i64,
        image_1:        Option<&str>,
        image_2:        Option<&str>,
        creation_at:    Option<i64>,
        customizing_id: Option<i64>,
        region:         Option<&str>,
        score:          Option<i64>,
    ) -> Result<bool> {
        let row = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scrapper_presets
                (id, class_id, title, user_nickname, character_name,
                 downloads, views, likes, image_1, image_2,
                 creation_at, customizing_id, region, score)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             ON CONFLICT (id) DO NOTHING
             RETURNING id",
        )
        .bind(id)
        .bind(class_id)
        .bind(title)
        .bind(user_nickname)
        .bind(character_name)
        .bind(downloads)
        .bind(views)
        .bind(likes)
        .bind(image_1)
        .bind(image_2)
        .bind(creation_at)
        .bind(customizing_id)
        .bind(region)
        .bind(score)
        .fetch_optional(pool)
        .await?;

        Ok(row.is_some())
    }

    /// Marks image slots as not_found when the preset page shows no image.
    /// Uses COALESCE so only non-None slots are updated.
    pub async fn update_image_names(
        pool:    &PgPool,
        id:      i64,
        image_1: Option<&str>,
        image_2: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE scrapper_presets
             SET image_1   = COALESCE($1, image_1),
                 image_2   = COALESCE($2, image_2),
                 updated_at = NOW()
             WHERE id = $3",
        )
        .bind(image_1)
        .bind(image_2)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Updates image URLs after a successful R2 upload.
    pub async fn update_image_urls(
        pool:       &PgPool,
        id:         i64,
        image_1_url: Option<&str>,
        image_2_url: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE scrapper_presets
             SET image_1_url = COALESCE($1, image_1_url),
                 image_2_url = COALESCE($2, image_2_url),
                 updated_at  = NOW()
             WHERE id = $3",
        )
        .bind(image_1_url)
        .bind(image_2_url)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// All existing preset IDs — used to pre-seed deduplication at session start.
    pub async fn get_all_ids(pool: &PgPool) -> Result<std::collections::HashSet<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM scrapper_presets")
            .fetch_all(pool)
            .await?;
        Ok(ids.into_iter().collect())
    }

    /// Top 10 pending image downloads per class, ordered by downloads desc.
    /// A preset is pending if it has image filenames but no URL uploaded yet (not marked not_found).
    pub async fn get_pending_downloads(
        pool: &PgPool,
    ) -> Result<Vec<(i64, i32, Option<String>, Option<String>, i64, i64, i64, Option<String>)>> {
        let rows = sqlx::query_as::<_, (i64, i32, Option<String>, Option<String>, i64, i64, i64, Option<String>)>(
            "SELECT id, class_id, image_1, image_2, downloads, views, likes, character_name
             FROM (
               SELECT *, ROW_NUMBER() OVER (PARTITION BY class_id ORDER BY downloads DESC) AS rn
               FROM scrapper_presets
               WHERE (image_1_url IS NULL AND image_1 IS NOT NULL AND image_1 != 'not_found')
                  OR (image_2_url IS NULL AND image_2 IS NOT NULL AND image_2 != 'not_found')
             ) t
             WHERE rn <= 10",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// Total presets + per-class breakdown for the Presets tab.
    pub async fn get_stats(pool: &PgPool) -> Result<serde_json::Value> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scrapper_presets")
            .fetch_one(pool).await?;

        let with_images: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM scrapper_presets WHERE image_1_url IS NOT NULL",
        ).fetch_one(pool).await?;

        let not_found: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM scrapper_presets WHERE image_1 = 'not_found'",
        ).fetch_one(pool).await?;

        let by_class: Vec<(i32, i64, i64)> = sqlx::query_as(
            "SELECT class_id, COUNT(*) AS total, COUNT(image_1_url) AS with_images
             FROM scrapper_presets GROUP BY class_id ORDER BY total DESC",
        ).fetch_all(pool).await?;

        Ok(serde_json::json!({
            "total":       total,
            "with_images": with_images,
            "not_found":   not_found,
            "pending":     total - with_images - not_found,
            "by_class":    by_class.into_iter().map(|(class_id, t, img)| serde_json::json!({
                "class_id":    class_id,
                "total":       t,
                "with_images": img,
            })).collect::<Vec<_>>(),
        }))
    }
}
