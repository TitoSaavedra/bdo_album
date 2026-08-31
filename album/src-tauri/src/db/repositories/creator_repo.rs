use sqlx::PgPool;

use crate::core::errors::Result;

pub struct CreatorRepository;

impl CreatorRepository {
    pub async fn get_favorites(pool: &PgPool) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT creator_nickname
            FROM album_creator_favorites
            ORDER BY creator_nickname
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn set_favorite(pool: &PgPool, creator_nickname: &str, is_fav: bool) -> Result<()> {
        if is_fav {
            sqlx::query(
                r#"
                INSERT INTO album_creator_favorites (creator_nickname)
                VALUES ($1)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(creator_nickname)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                DELETE FROM album_creator_favorites
                WHERE creator_nickname = $1
                "#,
            )
            .bind(creator_nickname)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
