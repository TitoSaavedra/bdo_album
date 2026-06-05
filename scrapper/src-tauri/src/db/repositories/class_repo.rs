use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(sqlx::FromRow)]
pub struct ClassRow {
    pub id:       i32,
    pub display:  String,
    pub icon_svg: Option<String>,
}

pub struct ClassRepository;

impl ClassRepository {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<ClassRow>> {
        let rows = sqlx::query_as::<_, ClassRow>(
            "SELECT id, display, icon_svg FROM scrapper_classes ORDER BY id",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
