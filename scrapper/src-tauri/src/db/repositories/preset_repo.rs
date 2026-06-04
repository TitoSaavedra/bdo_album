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
        creation_at:    Option<i64>,
        customizing_id: Option<i64>,
        region:         Option<&str>,
        score:          Option<i64>,
    ) -> Result<bool> {
        let row = sqlx::query_scalar::<_, i64>(
            "INSERT INTO scrapper_presets
                (id, class_id, title, user_nickname, character_name,
                 downloads, views, likes, creation_at, customizing_id, region, score)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
        .bind(creation_at)
        .bind(customizing_id)
        .bind(region)
        .bind(score)
        .fetch_optional(pool)
        .await?;

        Ok(row.is_some())
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
}
