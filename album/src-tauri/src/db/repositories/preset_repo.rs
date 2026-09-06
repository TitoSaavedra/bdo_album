use serde::Serialize;
use sqlx::PgPool;

use crate::core::errors::Result;

#[derive(Serialize, sqlx::FromRow)]
pub struct PresetRow {
    pub preset_id:      String,
    pub class_id:       i32,
    pub class_name:     String,
    pub title:          Option<String>,
    pub user_nickname:  Option<String>,
    pub character_name: Option<String>,
    pub region:         Option<String>,
    pub image_1_url:    Option<String>,
    pub image_2_url:    Option<String>,
    pub pab_url:        Option<String>,
    pub has_pab:        bool,
    pub has_modifications: bool,
    pub downloads:      Option<i64>,
    pub views:          Option<i64>,
    pub likes:          Option<i64>,
    pub is_wanted:      bool,
    pub is_discarded:   bool,
    pub creation_at:    Option<i64>,
    pub updated_at:     Option<i64>,
    pub auto_download_requested_at: Option<i64>,
    pub auto_download_error:        Option<String>,
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
    /// Unified browse query — replaces the old `get_by_class`/`get_by_creator` split.
    /// That split was *why* filters didn't compose: two separate functions with two
    /// separate parameter sets, so a creator filter structurally couldn't also apply
    /// region/day/etc. `class_ids` empty means "every class" (same shape the old
    /// creator-only path used); `creator` `None`/`""` means "any creator" — every
    /// filter is now just an optional narrowing condition on one query.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_filtered(
        pool:              &PgPool,
        class_ids:         &[i32],
        creator:           Option<&str>,
        offset:            i64,
        limit:             i64,
        sort_by:           &str,
        search:            &str,
        region:            Option<&str>,
        days_cutoff:       Option<i64>,
        has_modifications: Option<bool>,
        is_wanted:         Option<bool>,
        has_pab:           Option<bool>,
        show_discarded:    bool,
        r2_public_url:     &str,
    ) -> Result<Vec<PresetRow>> {
        let col = validated_sort(sort_by);
        // Built as a Rust-side conditional rather than a runtime
        // `cardinality($1) = 0 OR class_id = ANY($1)` check: the OR form
        // defeats `idx_presets_class(class_id, downloads DESC)` for the
        // common "exactly one class selected" case, since the planner can't
        // prove the class_id branch is the only live one without knowing the
        // bound value. Keeping `$1` referenced either way (as a no-op
        // `IS NOT NULL` check when empty) so the bind-parameter count still
        // lines up with the rest of the query.
        let class_clause = if class_ids.is_empty() {
            "$1::int[] IS NOT NULL"
        } else {
            "p.class_id = ANY($1::int[])"
        };
        let sql = format!(
            r#"
            SELECT
                p.id::TEXT                                         AS preset_id,
                p.class_id,
                c.display                                          AS class_name,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $6 || p.image_1_url                               AS image_1_url,
                $6 || p.image_2_url                               AS image_2_url,
                CASE WHEN pab.url IS NOT NULL THEN $6 || pab.url END AS pab_url,
                pab.url IS NOT NULL                               AS has_pab,
                EXISTS (SELECT 1 FROM album_preset_modifications m WHERE m.preset_id = p.id) AS has_modifications,
                p.downloads,
                p.views,
                p.likes,
                COALESCE(u.is_wanted,    false)                   AS is_wanted,
                COALESCE(u.is_discarded, false)                   AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT          AS updated_at,
                EXTRACT(EPOCH FROM u.auto_download_requested_at)::BIGINT AS auto_download_requested_at,
                u.auto_download_error
            FROM scraper_presets p
            JOIN scraper_classes c ON c.id = p.class_id
            LEFT JOIN album_user_prefs u ON u.preset_id = p.id
            LEFT JOIN LATERAL (
                SELECT url FROM scraper_preset_pabs
                WHERE preset_id = p.id
                ORDER BY id
                LIMIT 1
            ) pab ON true
            WHERE {class_clause}
              AND ($2 = '' OR p.user_nickname = $2)
              AND (p.image_1_url IS NOT NULL OR p.image_2_url IS NOT NULL)
              AND ($3 = '' OR (
                LOWER(COALESCE(p.title,          '')) LIKE '%' || LOWER($3) || '%' OR
                LOWER(COALESCE(p.user_nickname,  '')) LIKE '%' || LOWER($3) || '%' OR
                LOWER(COALESCE(p.character_name, '')) LIKE '%' || LOWER($3) || '%'
              ))
              AND ($7 = '' OR p.region = $7)
              AND ($8::BIGINT IS NULL OR (p.creation_at IS NOT NULL AND p.creation_at >= $8))
              AND ($9::BOOLEAN IS NOT TRUE OR EXISTS (
                SELECT 1 FROM album_preset_modifications m WHERE m.preset_id = p.id
              ))
              AND ($10::BOOLEAN IS NOT TRUE OR COALESCE(u.is_wanted, false) = true)
              AND ($11::BOOLEAN IS NOT TRUE OR pab.url IS NOT NULL)
              AND COALESCE(u.is_discarded, false) = $12
            ORDER BY
              (pab.url IS NOT NULL) DESC,
              COALESCE(u.is_wanted, false) DESC,
              p.{col} DESC NULLS LAST
            LIMIT $4 OFFSET $5
            "#
        );
        let rows = sqlx::query_as::<_, PresetRow>(&sql)
            .bind(class_ids)
            .bind(creator.unwrap_or(""))
            .bind(search)
            .bind(limit)
            .bind(offset)
            .bind(r2_public_url)
            .bind(region.unwrap_or(""))
            .bind(days_cutoff)
            .bind(has_modifications)
            .bind(is_wanted)
            .bind(has_pab)
            .bind(show_discarded)
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
                c.display                                          AS class_name,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $2 || p.image_1_url                               AS image_1_url,
                $2 || p.image_2_url                               AS image_2_url,
                CASE WHEN pab.url IS NOT NULL THEN $2 || pab.url END AS pab_url,
                pab.url IS NOT NULL                               AS has_pab,
                EXISTS (SELECT 1 FROM album_preset_modifications m WHERE m.preset_id = p.id) AS has_modifications,
                p.downloads,
                p.views,
                p.likes,
                COALESCE(u.is_wanted,    false)                   AS is_wanted,
                COALESCE(u.is_discarded, false)                   AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT          AS updated_at,
                EXTRACT(EPOCH FROM u.auto_download_requested_at)::BIGINT AS auto_download_requested_at,
                u.auto_download_error
            FROM scraper_presets p
            JOIN scraper_classes c ON c.id = p.class_id
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

    /// Queues presets for the scraper's auto-download worker (picked up from
    /// `album_user_prefs.auto_download_requested_at` — see scraper's `auto_download.rs`,
    /// which wakes up immediately via the `auto_download_queued` NOTIFY sent
    /// here instead of waiting on its fallback poll).
    /// Re-queueing clears a previous error so a failed item can be retried.
    pub async fn queue_auto_download(pool: &PgPool, preset_ids: &[i64]) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO album_user_prefs (preset_id, auto_download_requested_at)
            SELECT id, NOW() FROM UNNEST($1) AS id
            ON CONFLICT (preset_id) DO UPDATE
                SET auto_download_requested_at = NOW(),
                    auto_download_error        = NULL,
                    updated_at                 = NOW()
            "#,
        )
        .bind(preset_ids)
        .execute(pool)
        .await?;

        sqlx::query("SELECT pg_notify('auto_download_queued', $1)")
            .bind(serde_json::json!({ "preset_ids": preset_ids }).to_string())
            .execute(pool)
            .await?;
        Ok(())
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
                c.display                                                     AS class_name,
                p.title,
                p.user_nickname,
                p.character_name,
                p.region,
                $1 || p.image_1_url                                           AS image_1_url,
                $1 || p.image_2_url                                           AS image_2_url,
                NULL::TEXT                                                    AS pab_url,
                false                                                         AS has_pab,
                EXISTS (SELECT 1 FROM album_preset_modifications m WHERE m.preset_id = p.id) AS has_modifications,
                p.downloads,
                p.views,
                p.likes,
                true                                                          AS is_wanted,
                COALESCE(u.is_discarded, false)                               AS is_discarded,
                p.creation_at,
                EXTRACT(EPOCH FROM p.updated_at)::BIGINT                      AS updated_at,
                EXTRACT(EPOCH FROM u.auto_download_requested_at)::BIGINT      AS auto_download_requested_at,
                u.auto_download_error
            FROM album_user_prefs u
            JOIN scraper_presets p ON p.id = u.preset_id
            JOIN scraper_classes c ON c.id = p.class_id
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

    #[allow(clippy::too_many_arguments)]
    pub async fn count_by_search(
        pool:              &PgPool,
        search:            &str,
        region:            Option<&str>,
        days_cutoff:       Option<i64>,
        has_modifications: Option<bool>,
        is_wanted:         Option<bool>,
        has_pab:           Option<bool>,
        show_discarded:    bool,
    ) -> Result<Vec<(i32, i64)>> {
        let rows = sqlx::query_as::<_, (i32, i64)>(
            r#"
            SELECT p.class_id, COUNT(*)::BIGINT
            FROM scraper_presets p
            LEFT JOIN album_user_prefs u ON u.preset_id = p.id
            WHERE (p.image_1_url IS NOT NULL OR p.image_2_url IS NOT NULL)
              AND ($1 = '' OR (
                LOWER(COALESCE(p.title,          '')) LIKE '%' || LOWER($1) || '%' OR
                LOWER(COALESCE(p.user_nickname,  '')) LIKE '%' || LOWER($1) || '%' OR
                LOWER(COALESCE(p.character_name, '')) LIKE '%' || LOWER($1) || '%'
              ))
              AND ($2 = '' OR p.region = $2)
              AND ($3::BIGINT IS NULL OR (p.creation_at IS NOT NULL AND p.creation_at >= $3))
              AND ($4::BOOLEAN IS NOT TRUE OR EXISTS (
                SELECT 1 FROM album_preset_modifications m WHERE m.preset_id = p.id
              ))
              AND ($5::BOOLEAN IS NOT TRUE OR COALESCE(u.is_wanted, false) = true)
              AND ($6::BOOLEAN IS NOT TRUE OR EXISTS (
                SELECT 1 FROM scraper_preset_pabs WHERE preset_id = p.id
              ))
              AND COALESCE(u.is_discarded, false) = $7
            GROUP BY p.class_id
            "#,
        )
        .bind(search)
        .bind(region.unwrap_or(""))
        .bind(days_cutoff)
        .bind(has_modifications)
        .bind(is_wanted)
        .bind(has_pab)
        .bind(show_discarded)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
