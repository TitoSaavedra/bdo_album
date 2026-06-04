use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sqlx::PgPool;
use tauri::AppHandle;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::core::errors::{AppError, Result};
use crate::db::repositories::{
    class_repo::ClassRepository,
    log_repo::LogRepository,
    preset_repo::PresetRepository,
    session_repo::SessionRepository,
};
use crate::events::{ClassStatsUpdated, Events, FetchProgress, ImageProgress, PresetSynced, UploadProgress};

use super::browser::BrowserSession;
use super::garmoth::{GarmothClient, GarmothPreset};
use super::r2::R2Client;

const REGIONS: &[&str] = &["eu", "na", "ru", "jp", "kr", "tw", "sa", "sea", "asia", "mena"];
const DAYS:    &str    = "ever";

// ── Types ─────────────────────────────────────────────────────

pub struct ImageTask {
    pub preset_id:  i64,
    pub class_id:   i32,
    pub class_name: String,
    pub image_1:    Option<String>,    // raw filename
    pub image_2:    Option<String>,
    pub image_1_url: Option<String>,   // Garmoth CDN URL
    pub image_2_url: Option<String>,
}

pub struct FetchResult {
    pub total_fetched: usize,
    pub total_errors:  usize,
    pub image_tasks:   Vec<ImageTask>,
}

pub struct DownloadResult {
    pub total_images:   usize,
    pub total_uploaded: usize,
    pub total_errors:   usize,
}

// ── Phase 1: Fetch JSON ───────────────────────────────────────

pub async fn run_fetch(
    app:         &AppHandle,
    pool:        &PgPool,
    cf_token:    String,
    session_id:  i64,
    cancel:      Arc<AtomicBool>,
    parallelism: usize,
) -> Result<FetchResult> {
    let classes = ClassRepository::get_all(pool).await?;
    let total   = classes.len();
    let client  = Arc::new(GarmothClient::new(&cf_token));

    let mut total_fetched = 0usize;
    let mut total_errors  = 0usize;
    let mut image_tasks: Vec<ImageTask> = Vec::new();

    for (i, class) in classes.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) { break; }

        Events::fetch_progress(app, FetchProgress {
            class_id:   class.id as u32,
            class_name: class.display.clone(),
            fetched:    i,
            total,
        });

        let class_id = class.id as u32;
        let sem = Arc::new(Semaphore::new(parallelism));
        let mut join_set: JoinSet<(&'static str, std::result::Result<Vec<GarmothPreset>, AppError>)> =
            JoinSet::new();

        for &region in REGIONS {
            if cancel.load(Ordering::Relaxed) { break; }
            let client = client.clone();
            let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
            join_set.spawn(async move {
                let result = client.fetch_popular(class_id, DAYS, region).await;
                drop(permit);
                (region, result)
            });
        }

        let mut seen: HashSet<i64> = HashSet::new();
        let mut class_fetched = 0usize;
        let mut class_errors  = 0usize;

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((region, Ok(presets))) => {
                    for p in &presets {
                        if !seen.insert(p.id) { continue; }

                        match PresetRepository::insert_new(
                            pool, p.id, p.class_id,
                            p.title.as_deref(), p.user_nickname.as_deref(),
                            p.character_name.as_deref(),
                            p.downloads, p.views, p.likes,
                            p.creation_at, p.customizing_id,
                            p.region.as_deref(), p.score,
                        ).await {
                            Ok(true) => {
                                class_fetched += 1;

                                if p.image_1.is_some() || p.image_2.is_some() {
                                    image_tasks.push(ImageTask {
                                        preset_id:   p.id,
                                        class_id:    p.class_id,
                                        class_name:  class.display.clone(),
                                        image_1:     p.image_1.clone(),
                                        image_2:     p.image_2.clone(),
                                        image_1_url: p.image_1_url.clone(),
                                        image_2_url: p.image_2_url.clone(),
                                    });
                                }

                                Events::preset_synced(app, PresetSynced {
                                    preset_id:   p.id.to_string(),
                                    class_id:    p.class_id as u32,
                                    image_1_url: None,
                                    image_2_url: None,
                                });
                            }
                            Ok(false) => {} // already existed
                            Err(e) => {
                                class_errors += 1;
                                LogRepository::insert(
                                    app, pool, Some(session_id), "error", "insert_new",
                                    &format!("preset {} failed: {}", p.id, e),
                                ).await.ok();
                            }
                        }
                    }
                }
                Ok((region, Err(e))) => {
                    class_errors += 1;
                    LogRepository::insert(
                        app, pool, Some(session_id), "error", "fetch_popular",
                        &format!("class={} region={} error: {}", class.display, region, e),
                    ).await.ok();
                }
                Err(_) => { class_errors += 1; }
            }
        }

        total_fetched += class_fetched;
        total_errors  += class_errors;

        SessionRepository::upsert_class_stats(
            pool, session_id, class.id, class_fetched as i32, 0, class_errors as i32,
        ).await.ok();

        Events::class_stats_updated(app, ClassStatsUpdated {
            class_id: class.id as u32,
            fetched:  class_fetched,
            images:   0,
            errors:   class_errors,
        });

        Events::fetch_progress(app, FetchProgress {
            class_id:   class.id as u32,
            class_name: class.display.clone(),
            fetched:    i + 1,
            total,
        });
    }

    Ok(FetchResult { total_fetched, total_errors, image_tasks })
}

// ── Phase 2: Download from Garmoth CDN + Upload to R2 ────────

pub async fn run_download_upload(
    app:        &AppHandle,
    browser:    &BrowserSession,
    r2:         &R2Client,
    pool:       &PgPool,
    session_id: i64,
    cancel:     Arc<AtomicBool>,
    tasks:      Vec<ImageTask>,
) -> Result<DownloadResult> {
    // Count total images to download for progress
    let total_images: usize = tasks.iter().map(|t| {
        t.image_1_url.is_some() as usize + t.image_2_url.is_some() as usize
    }).sum();

    let mut done      = 0usize;
    let mut uploaded  = 0usize;
    let mut errors    = 0usize;

    for task in &tasks {
        if cancel.load(Ordering::Relaxed) { break; }

        let mut r2_url_1: Option<String> = None;
        let mut r2_url_2: Option<String> = None;

        // ── Image 1 ──
        if let (Some(cdn_url), Some(filename)) = (&task.image_1_url, &task.image_1) {
            match download_and_upload(
                app, browser, r2,
                cdn_url, filename,
                &task.class_name, task.preset_id,
                1, done, total_images,
            ).await {
                Ok(url) => { r2_url_1 = Some(url); uploaded += 1; }
                Err(e)  => {
                    errors += 1;
                    LogRepository::insert(
                        app, pool, Some(session_id), "error", "img_upload",
                        &format!("preset {} img1: {}", task.preset_id, e),
                    ).await.ok();
                }
            }
            done += 1;
            Events::image_progress(app, ImageProgress {
                preset_id:  task.preset_id.to_string(),
                class_name: task.class_name.clone(),
                image_num:  1,
                done,
                total: total_images,
            });
        }

        // ── Image 2 ──
        if let (Some(cdn_url), Some(filename)) = (&task.image_2_url, &task.image_2) {
            match download_and_upload(
                app, browser, r2,
                cdn_url, filename,
                &task.class_name, task.preset_id,
                2, done, total_images,
            ).await {
                Ok(url) => { r2_url_2 = Some(url); uploaded += 1; }
                Err(e)  => {
                    errors += 1;
                    LogRepository::insert(
                        app, pool, Some(session_id), "error", "img_upload",
                        &format!("preset {} img2: {}", task.preset_id, e),
                    ).await.ok();
                }
            }
            done += 1;
            Events::image_progress(app, ImageProgress {
                preset_id:  task.preset_id.to_string(),
                class_name: task.class_name.clone(),
                image_num:  2,
                done,
                total: total_images,
            });
        }

        // Update DB with R2 URLs and emit preset_synced
        if r2_url_1.is_some() || r2_url_2.is_some() {
            PresetRepository::update_image_urls(
                pool, task.preset_id,
                r2_url_1.as_deref(), r2_url_2.as_deref(),
            ).await.ok();

            Events::preset_synced(app, PresetSynced {
                preset_id:   task.preset_id.to_string(),
                class_id:    task.class_id as u32,
                image_1_url: r2_url_1,
                image_2_url: r2_url_2,
            });
        }
    }

    Ok(DownloadResult { total_images: done, total_uploaded: uploaded, total_errors: errors })
}

// ── Helper ────────────────────────────────────────────────────

async fn download_and_upload(
    app:        &AppHandle,
    browser:    &BrowserSession,
    r2:         &R2Client,
    cdn_url:    &str,
    filename:   &str,
    class_name: &str,
    preset_id:  i64,
    img_num:    u8,
    done:       usize,
    total:      usize,
) -> Result<String> {
    let bytes = browser.download(cdn_url).await?;

    Events::upload_progress(app, crate::events::UploadProgress {
        preset_id: preset_id.to_string(),
        image_url: cdn_url.to_string(),
        done,
        total,
    });

    let key = format!("images/{}/{}/{}", class_name, preset_id, filename);
    r2.upload(&key, bytes).await
}
