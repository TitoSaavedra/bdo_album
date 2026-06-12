use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FaceGridRow {
    pub id:            i64,
    pub name:          String,
    pub account_id:    String,
    pub thumbnail_url: Option<String>,
    pub created_at:    i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FaceGridSlotRow {
    pub id:           i64,
    pub grid_id:      i64,
    pub character_no: String,
    pub image_url:    String,
    pub slot_order:   i32,
}

pub struct FaceGridRepository;

impl FaceGridRepository {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<FaceGridRow>> {
        let rows = sqlx::query_as!(
            FaceGridRow,
            r#"SELECT
                id,
                name,
                account_id,
                thumbnail_url,
                EXTRACT(EPOCH FROM created_at)::BIGINT AS "created_at!"
               FROM album_face_grids
               ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn create(
        pool:          &PgPool,
        name:          &str,
        account_id:    &str,
        thumbnail_url: Option<&str>,
    ) -> Result<FaceGridRow> {
        let row = sqlx::query_as!(
            FaceGridRow,
            r#"INSERT INTO album_face_grids (name, account_id, thumbnail_url)
               VALUES ($1, $2, $3)
               RETURNING
                id,
                name,
                account_id,
                thumbnail_url,
                EXTRACT(EPOCH FROM created_at)::BIGINT AS "created_at!""#,
            name,
            account_id,
            thumbnail_url,
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn delete(pool: &PgPool, grid_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM album_face_grids WHERE id = $1", grid_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn upsert_slot(
        pool:         &PgPool,
        grid_id:      i64,
        character_no: &str,
        image_url:    &str,
        slot_order:   i32,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO album_face_grid_slots (grid_id, character_no, image_url, slot_order)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (grid_id, character_no)
               DO UPDATE SET image_url = EXCLUDED.image_url, slot_order = EXCLUDED.slot_order"#,
        )
        .bind(grid_id)
        .bind(character_no)
        .bind(image_url)
        .bind(slot_order)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_slots(pool: &PgPool, grid_id: i64, r2_public_url: &str) -> Result<Vec<FaceGridSlotRow>> {
        let rows = sqlx::query_as::<_, (i64, i64, String, String, i32)>(
            "SELECT id, grid_id, character_no, image_url, slot_order FROM album_face_grid_slots WHERE grid_id = $1 ORDER BY slot_order"
        )
        .bind(grid_id)
        .fetch_all(pool)
        .await?;

        // Prepend r2_public_url to image_url
        Ok(rows.into_iter().map(|(id, grid_id, character_no, image_url, slot_order)| {
            let full_url = if image_url.is_empty() {
                image_url
            } else {
                format!("{}/{}", r2_public_url, image_url)
            };
            FaceGridSlotRow {
                id,
                grid_id,
                character_no,
                image_url: full_url,
                slot_order,
            }
        }).collect())
    }

    pub async fn get_all_character_faces(pool: &PgPool) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT character_no, image_url FROM album_character_faces ORDER BY updated_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
