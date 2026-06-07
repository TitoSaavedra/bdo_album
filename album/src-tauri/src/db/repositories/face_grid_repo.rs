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
    pub preset_id:    Option<i64>,
    pub slot_order:   i32,
    pub image_1_url:  Option<String>,
    pub preset_title: Option<String>,
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
        preset_id:    Option<i64>,
        slot_order:   i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO album_face_grid_slots (grid_id, character_no, preset_id, slot_order)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (grid_id, character_no)
               DO UPDATE SET preset_id = EXCLUDED.preset_id, slot_order = EXCLUDED.slot_order"#,
            grid_id,
            character_no,
            preset_id,
            slot_order,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_slots(
        pool:          &PgPool,
        grid_id:       i64,
        r2_public_url: &str,
    ) -> Result<Vec<FaceGridSlotRow>> {
        let rows = sqlx::query_as!(
            FaceGridSlotRow,
            r#"SELECT
                s.id,
                s.grid_id,
                s.character_no,
                s.preset_id,
                s.slot_order,
                CASE WHEN p.image_1_url IS NOT NULL
                     THEN $2 || p.image_1_url
                END AS image_1_url,
                p.title AS preset_title
               FROM album_face_grid_slots s
               LEFT JOIN scrapper_presets p ON p.id = s.preset_id
               WHERE s.grid_id = $1
               ORDER BY s.slot_order"#,
            grid_id,
            r2_public_url,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
