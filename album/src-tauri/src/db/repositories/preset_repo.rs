use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(Serialize, sqlx::FromRow)]
pub struct PresetRow {
    pub preset_id:      String,
    pub class_id:       i32,
    pub title:          Option<String>,
    pub user_nickname:  Option<String>,
    pub character_name: Option<String>,
    pub region:         Option<String>,
    pub image_1_url:    Option<String>,
    pub image_2_url:    Option<String>,
    pub pab_url:        Option<String>,
    pub has_pab:        bool,
    pub downloads:      Option<i64>,
    pub views:          Option<i64>,
    pub likes:          Option<i64>,
    pub is_wanted:      bool,
    pub is_discarded:   bool,
    pub creation_at:    Option<i64>,
    pub updated_at:     Option<i64>,
}

fn validated_sort(sort_by: &str) -> &'static str {
    match sort_by {
        "views" => "views",
        "likes" => "likes",
        _       => "downloads",
    }
}

pub struct PresetRepository;

impl PresetRepository {
    pub async fn get_by_class(
        pool:          &PgPool,
        class_id:      i32,
        offset:        i64,
        limit:         i64,
        sort_by:       &str,
        search:        &str,
        region:        Option<&str>,
        days_cutoff:   Option<i64>,
        r2_public_url: &str,
    ) -> Result<Vec<PresetRow>> {
        let col = validated_sort(sort_by);
        let sql = format!(
            r#"
            SELECT
                p.id::TEXT                                         AS preset_id,
                p.class_id,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $5 || p.image_1_url                               AS image_1_url,
                $5 || p.image_2_url                               AS image_2_url,
                CASE WHEN pab.url IS NOT NULL THEN $5 || pab.url END AS pab_url,
                pab.url IS NOT NULL                               AS has_pab,
                p.downloads,
                p.views,
                p.likes,
                COALESCE(u.is_wanted,    false)                   AS is_wanted,
                COALESCE(u.is_discarded, false)                   AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT          AS updated_at
            FROM scraper_presets p
            LEFT JOIN album_user_prefs u ON u.preset_id = p.id
            LEFT JOIN LATERAL (
                SELECT url FROM scraper_preset_pabs
                WHERE preset_id = p.id
                ORDER BY id
                LIMIT 1
            ) pab ON true
            WHERE p.class_id = $1
              AND (p.image_1_url IS NOT NULL OR p.image_2_url IS NOT NULL)
              AND COALESCE(u.is_discarded, false) = false
              AND ($2 = '' OR (
                LOWER(COALESCE(p.title,          '')) LIKE '%' || LOWER($2) || '%' OR
                LOWER(COALESCE(p.user_nickname,  '')) LIKE '%' || LOWER($2) || '%' OR
                LOWER(COALESCE(p.character_name, '')) LIKE '%' || LOWER($2) || '%'
              ))
              AND ($6 = '' OR p.region = $6)
              AND ($7::BIGINT IS NULL OR (p.creation_at IS NOT NULL AND p.creation_at >= $7))
            ORDER BY
              (pab.url IS NOT NULL) DESC,
              COALESCE(u.is_wanted, false) DESC,
              p.{col} DESC NULLS LAST
            LIMIT $3 OFFSET $4
            "#
        );
        let rows = sqlx::query_as::<_, PresetRow>(&sql)
            .bind(class_id)
            .bind(search)
            .bind(limit)
            .bind(offset)
            .bind(r2_public_url)
            .bind(region.unwrap_or(""))
            .bind(days_cutoff)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_by_id(
        pool:          &PgPool,
        preset_id:     i64,
        r2_public_url: &str,
    ) -> Result<Option<PresetRow>> {
        let row = sqlx::query_as::<_, PresetRow>(
            r#"
            SELECT
                p.id::TEXT                                         AS preset_id,
                p.class_id,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $2 || p.image_1_url                               AS image_1_url,
                $2 || p.image_2_url                               AS image_2_url,
                CASE WHEN pab.url IS NOT NULL THEN $2 || pab.url END AS pab_url,
                pab.url IS NOT NULL                               AS has_pab,
                p.downloads,
                p.views,
                p.likes,
                COALESCE(u.is_wanted,    false)                   AS is_wanted,
                COALESCE(u.is_discarded, false)                   AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT          AS updated_at
            FROM scraper_presets p
            LEFT JOIN album_user_prefs u ON u.preset_id = p.id
            LEFT JOIN LATERAL (
                SELECT url FROM scraper_preset_pabs
                WHERE preset_id = p.id
                ORDER BY id
                LIMIT 1
            ) pab ON true
            WHERE p.id = $1
            "#,
        )
        .bind(preset_id)
        .bind(r2_public_url)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    pub async fn get_distinct_regions(pool: &PgPool) -> Result<Vec<String>> {
        let regions = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT region
            FROM scraper_presets
            WHERE region IS NOT NULL AND region != ''
              AND (image_1_url IS NOT NULL OR image_2_url IS NOT NULL)
            ORDER BY region
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(regions)
    }

    pub async fn get_class_id(pool: &PgPool, class_name: &str) -> Result<Option<i32>> {
        let id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM scraper_classes WHERE display = $1",
        )
        .bind(class_name)
        .fetch_optional(pool)
        .await?;
        Ok(id)
    }

    pub async fn upsert_discard(pool: &PgPool, preset_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_user_prefs (preset_id, is_discarded)
            VALUES ($1, true)
            ON CONFLICT (preset_id) DO UPDATE
                SET is_discarded = true, updated_at = NOW()
            "#,
        )
        .bind(preset_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn toggle_wanted(pool: &PgPool, preset_id: i64) -> Result<bool> {
        let new_val: bool = sqlx::query_scalar(
            r#"
            INSERT INTO album_user_prefs (preset_id, is_wanted)
            VALUES ($1, true)
            ON CONFLICT (preset_id) DO UPDATE
                SET is_wanted  = NOT album_user_prefs.is_wanted,
                    updated_at = NOW()
            RETURNING is_wanted
            "#,
        )
        .bind(preset_id)
        .fetch_one(pool)
        .await?;
        Ok(new_val)
    }

    pub async fn get_wanted_ids(pool: &PgPool) -> Result<Vec<String>> {
        let ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT u.preset_id::TEXT
            FROM album_user_prefs u
            WHERE u.is_wanted = true
              AND NOT EXISTS (
                SELECT 1 FROM scraper_preset_pabs WHERE preset_id = u.preset_id
              )
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(ids)
    }

    pub async fn get_wanted_pab_urls(pool: &PgPool) -> Result<Vec<String>> {
        let urls = sqlx::query_scalar::<_, String>(
            r#"
            SELECT 'https://garmoth.com/beauty-album/preset/' || p.id::TEXT
            FROM album_user_prefs u
            JOIN scraper_presets p ON p.id = u.preset_id
            WHERE u.is_wanted = true
              AND NOT EXISTS (
                SELECT 1 FROM scraper_preset_pabs WHERE preset_id = p.id
              )
            ORDER BY p.id
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(urls)
    }

    pub async fn get_wanted_presets(pool: &PgPool, r2_public_url: &str) -> Result<Vec<PresetRow>> {
        let rows = sqlx::query_as::<_, PresetRow>(
            r#"
            SELECT
                p.id::TEXT                                                    AS preset_id,
                p.class_id,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $1 || p.image_1_url                                           AS image_1_url,
                $1 || p.image_2_url                                           AS image_2_url,
                NULL::TEXT                                                    AS pab_url,
                false                                                         AS has_pab,
                p.downloads,
                p.views,
                p.likes,
                true                                                          AS is_wanted,
                COALESCE(u.is_discarded, false)                               AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT                      AS updated_at
            FROM album_user_prefs u
            JOIN scraper_presets p ON p.id = u.preset_id
            WHERE u.is_wanted = true
              AND NOT EXISTS (
                SELECT 1 FROM scraper_preset_pabs WHERE preset_id = p.id
              )
            ORDER BY p.id
            "#,
        )
        .bind(r2_public_url)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn count_by_search(
        pool:        &PgPool,
        search:      &str,
        region:      Option<&str>,
        days_cutoff: Option<i64>,
    ) -> Result<Vec<(i32, i64)>> {
        let rows = sqlx::query_as::<_, (i32, i64)>(
            r#"
            SELECT p.class_id, COUNT(*)::BIGINT
            FROM scraper_presets p
            LEFT JOIN album_user_prefs u ON u.preset_id = p.id
            WHERE (p.image_1_url IS NOT NULL OR p.image_2_url IS NOT NULL)
              AND COALESCE(u.is_discarded, false) = false
              AND ($1 = '' OR (
                LOWER(COALESCE(p.title,          '')) LIKE '%' || LOWER($1) || '%' OR
                LOWER(COALESCE(p.user_nickname,  '')) LIKE '%' || LOWER($1) || '%' OR
                LOWER(COALESCE(p.character_name, '')) LIKE '%' || LOWER($1) || '%'
              ))
              AND ($2 = '' OR p.region = $2)
              AND ($3::BIGINT IS NULL OR (p.creation_at IS NOT NULL AND p.creation_at >= $3))
            GROUP BY p.class_id
            "#,
        )
        .bind(search)
        .bind(region.unwrap_or(""))
        .bind(days_cutoff)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
