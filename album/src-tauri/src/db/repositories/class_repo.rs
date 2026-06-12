use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(Serialize, sqlx::FromRow)]
pub struct ClassRow {
    pub class_id:     i32,
    pub name:         String,
    pub icon_svg:     Option<String>,
    pub preset_count: i64,
    pub is_favorite:  bool,
}

pub struct ClassRepository;

impl ClassRepository {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<ClassRow>> {
        let rows = sqlx::query_as::<_, ClassRow>(
            r#"
            SELECT
                c.id                                                     AS class_id,
                c.display                                                AS name,
                c.icon_svg,
                COUNT(p.id)                                              AS preset_count,
                EXISTS(
                    SELECT 1 FROM album_class_favorites f
                    WHERE f.class_id = c.id
                )                                                        AS is_favorite
            FROM scraper_classes c
            LEFT JOIN scraper_presets p ON p.class_id = c.id
                AND (p.image_1_url IS NOT NULL OR p.image_2_url IS NOT NULL)
            GROUP BY c.id, c.display, c.icon_svg
            ORDER BY is_favorite DESC, preset_count DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_favorites(pool: &PgPool) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT c.display
            FROM scraper_classes c
            JOIN album_class_favorites f ON f.class_id = c.id
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn set_favorite(pool: &PgPool, class_name: &str, is_fav: bool) -> Result<()> {
        if is_fav {
            sqlx::query(
                r#"
                INSERT INTO album_class_favorites (class_id)
                SELECT id FROM scraper_classes WHERE display = $1
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(class_name)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                DELETE FROM album_class_favorites
                WHERE class_id = (SELECT id FROM scraper_classes WHERE display = $1)
                "#,
            )
            .bind(class_name)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
