use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use sqlx::PgPool;
use tauri::{AppHandle, Manager};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

use crate::core::errors::{AppError, Result};
use crate::core::state::AppState;
use crate::db::repositories::{
    class_repo::ClassRepository,
    log_repo::LogRepository,
    preset_repo::PresetRepository,
    session_repo::SessionRepository,
};
use crate::events::{
    ClassStatsUpdated, Events, FetchProgress, ImageProgress, ProgressStatus, ProgressType,
    PresetSynced, ScrapperDone, ScrapperError, ScrapperPhase, ScrapperProgress, UploadProgress,
};

use super::browser::BrowserSession;
use super::garmoth::{GarmothClient, GarmothPreset};
use super::r2::R2Client;

// ── Types ─────────────────────────────────────────────────────

pub struct ImageTask {
    pub preset_id:      i64,
    pub class_id:       i32,
    pub class_name:     String,
    pub image_1:        Option<String>,
    pub image_2:        Option<String>,
    pub downloads:      i64,
    pub views:          i64,
    pub likes:          i64,
    pub character_name: Option<String>,
}

pub struct FetchResult {
    pub total_fetched:  usize,
    pub total_errors:   usize,
    pub total_skipped:  usize,
}

pub struct DownloadResult {
    pub total_images:   usize,
    pub total_uploaded: usize,
    pub total_errors:   usize,
}

// ── Session Orchestration ─────────────────────────────────────

pub async fn run_session(
    app:         AppHandle,
    pool:        PgPool,
    cancel:      Arc<AtomicBool>,
    session_id:  i64,
    parallelism: usize,
    days:        Vec<String>,
    regions:     Vec<String>,
    classes:     Vec<serde_json::Value>,
) {
    let started = Instant::now();

    let r2 = match R2Client::from_env() {
        Ok(r2) => r2,
        Err(e) => { abort_session(&app, &pool, session_id, e).await; return; }
    };

    Events::sync_loading(&app, "Starting browser");
    LogRepository::insert(&app, &pool, Some(session_id), "INFO", "browser", "Starting browser").await.ok();
    let browser = Arc::new(match BrowserSession::new(&app, &pool, session_id).await {
        Ok(b) => b,
        Err(e) => { abort_session(&app, &pool, session_id, e).await; return; }
    });
    LogRepository::insert(&app, &pool, Some(session_id), "INFO", "browser", "Browser ready").await.ok();

    Events::sync_loading(&app, "Waiting for CF clearance");
    LogRepository::insert(&app, &pool, Some(session_id), "INFO", "browser", "Waiting for CF clearance (up to 30s)").await.ok();
    let cf_token = browser.wait_for_cf_clearance(30).await.unwrap_or_default();
    LogRepository::insert(&app, &pool, Some(session_id), "INFO", "browser",
        if cf_token.is_empty() { "CF clearance not found — proceeding without it" } else { "CF clearance obtained" },
    ).await.ok();

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "fetch", "Fetch phase started").await.ok();
    Events::sync_loading(&app, "Fetching & downloading...");

    let img_total = Arc::new(AtomicUsize::new(0));
    let img_done  = Arc::new(AtomicUsize::new(0));
    let (tx, rx)  = mpsc::channel::<ImageTask>(512);
    let tx_pending = tx.clone();

    // Fetch, download, and pending-image loop run concurrently.
    // Fetch sends new presets; pending loop sends DB presets awaiting images; download consumes both.
    let (fetch_result, dl, _) = tokio::join!(
        run_fetch(&app, &pool, cf_token, session_id, cancel.clone(), parallelism, days, regions, classes, tx, img_total.clone()),
        run_download_pipeline(&app, Arc::clone(&browser), &r2, &pool, session_id, cancel.clone(), img_done, img_total.clone(), rx, parallelism),
        run_pending_loop(&app, &pool, session_id, cancel.clone(), tx_pending, img_total),
    );

    let fetch = match fetch_result {
        Ok(r) => r,
        Err(e) => { abort_session(&app, &pool, session_id, e).await; return; }
    };

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "fetch",
        &format!("Fetch done — {} new presets, {} errors", fetch.total_fetched, fetch.total_errors),
    ).await.ok();

    LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "images",
        &format!("Images done — {} done, {} uploaded, {} errors", dl.total_images, dl.total_uploaded, dl.total_errors),
    ).await.ok();

    let elapsed      = started.elapsed().as_secs();
    let cancelled    = cancel.load(Ordering::Relaxed);
    let total_errors = fetch.total_errors + dl.total_errors;
    let status       = if cancelled { "cancelled" } else { "done" };

    finish_session(&app, &pool, session_id, status, fetch.total_fetched, dl.total_images, dl.total_uploaded, total_errors, fetch.total_skipped, started).await;

    if cancelled {
        LogRepository::insert(&app, &pool, Some(session_id), "WARN", "session", "Session cancelled").await.ok();
        Events::scrapper_cancelled(&app);
    } else {
        LogRepository::insert(&app, &pool, Some(session_id), "ORCH", "session",
            &format!("Session #{} done in {}s — {} presets, {} images, {} errors",
                session_id, elapsed, fetch.total_fetched, dl.total_uploaded, total_errors),
        ).await.ok();
        Events::scrapper_done(&app, ScrapperDone {
            total_fetched:  fetch.total_fetched,
            total_images:   dl.total_images,
            total_uploaded: dl.total_uploaded,
            errors:         total_errors,
            elapsed_secs:   elapsed,
        });
    }
}

// ── Phase: Fetch ──────────────────────────────────────────────

pub async fn run_fetch(
    app:            &AppHandle,
    pool:           &PgPool,
    cf_token:       String,
    session_id:     i64,
    cancel:         Arc<AtomicBool>,
    parallelism:    usize,
    days:           Vec<String>,
    regions:        Vec<String>,
    classes_filter: Vec<serde_json::Value>,
    tx:             mpsc::Sender<ImageTask>,
    img_total:      Arc<AtomicUsize>,
) -> Result<FetchResult> {
    let all_db_classes = ClassRepository::get_all(pool).await?;
    let client = Arc::new(GarmothClient::new(&cf_token));

    // Map DB class IDs → display names, used to resolve preset.class_id from API responses
    let class_name_map: std::collections::HashMap<i32, String> = all_db_classes.iter()
        .map(|c| (c.id, c.display.clone()))
        .collect();

    // Build class entries from the filter.
    // "all" → class=None (global ranking endpoint), numbers → class=Some(id).
    // Both are treated as single options: each generates days × regions requests.
    struct ClassEntry { garmoth_id: Option<u32>, db_id: i32, name: String }
    let mut class_entries: Vec<ClassEntry> = Vec::new();
    for v in &classes_filter {
        if v.as_str() == Some("all") {
            class_entries.push(ClassEntry { garmoth_id: None, db_id: -1, name: "All classes".to_string() });
        } else if let Some(n) = v.as_i64() {
            let id = n as i32;
            if let Some(c) = all_db_classes.iter().find(|c| c.id == id) {
                class_entries.push(ClassEntry { garmoth_id: Some(id as u32), db_id: id, name: c.display.clone() });
            }
        }
    }

    let existing_ids: HashSet<i64> = PresetRepository::get_all_ids(pool).await.unwrap_or_default();
    let mut global_seen = existing_ids;
    LogRepository::insert(app, pool, Some(session_id), "ORCH", "fetch",
        &format!("Pre-loaded {} existing preset IDs from DB", global_seen.len()),
    ).await.ok();

    let total_classes  = class_entries.len();
    let total_requests = total_classes * days.len() * regions.len();
    LogRepository::insert(app, pool, Some(session_id), "ORCH", "fetch",
        &format!("{} classes × {} days × {} regions = {} requests total",
            total_classes, days.len(), regions.len(), total_requests),
    ).await.ok();

    // Flatten all work: day → region → class order so classes interleave from the start.
    struct WorkItem { db_id: i32, garmoth_id: Option<u32>, label: String, d: String, r: String }
    let mut work: Vec<WorkItem> = Vec::with_capacity(total_requests);
    for day in &days {
        for region in &regions {
            for class in &class_entries {
                work.push(WorkItem {
                    db_id:      class.db_id,
                    garmoth_id: class.garmoth_id,
                    label:      format!("{}/{}/{}", class.name, day, region),
                    d:          day.clone(),
                    r:          region.clone(),
                });
            }
        }
    }

    let mut total_fetched = 0usize;
    let mut total_errors  = 0usize;
    let mut total_skipped = 0usize;
    let mut class_stats: std::collections::HashMap<i32, (usize, usize, usize)> = std::collections::HashMap::new();

    // Process in batches of `parallelism`:
    //   1. Refresh global_seen from DB
    //   2. Spawn batch in parallel
    //   3. Drain + validate + insert
    for chunk in work.chunks(parallelism) {
        if cancel.load(Ordering::Relaxed) { break; }

        // Refresh before spawning so this batch skips IDs inserted by previous batches
        if let Ok(fresh_ids) = PresetRepository::get_all_ids(pool).await {
            let before = global_seen.len();
            global_seen.extend(fresh_ids);
            let added = global_seen.len() - before;
            LogRepository::insert(app, pool, Some(session_id), "ORCH", "fetch",
                &format!("DB refresh: +{} IDs ({} total seen)", added, global_seen.len()),
            ).await.ok();
        }

        let mut js: JoinSet<(i32, String, std::result::Result<Vec<GarmothPreset>, AppError>)> = JoinSet::new();

        for item in chunk {
            let client = client.clone();
            let db_id  = item.db_id;
            let gid    = item.garmoth_id;
            let label  = item.label.clone();
            let d      = item.d.clone();
            let r      = item.r.clone();
            LogRepository::insert(app, pool, Some(session_id), "FETCH", "fetch",
                &format!("→ {}", label),
            ).await.ok();
            js.spawn(async move {
                let res = client.fetch_popular(gid, &d, &r).await;
                (db_id, label, res)
            });
        }

        let mut batch_total_new     = 0usize;
        let mut batch_total_skipped = 0usize;

        while let Some(res) = js.join_next().await {
            let (db_id, label, result) = match res { Ok(v) => v, Err(_) => { total_errors += 1; continue; } };
            let stat = class_stats.entry(db_id).or_insert((0, 0, 0));
            match result {
                Ok(presets) => {
                    let mut batch_new     = 0usize;
                    let mut batch_skipped = 0usize;
                    for p in &presets {
                        if !global_seen.insert(p.id) {
                            stat.1 += 1; total_skipped += 1; batch_skipped += 1;
                            continue;
                        }
                        let (fetched, errors) = insert_preset_and_queue(
                            app, pool, session_id, &tx, &img_total,
                            p, class_name_map.get(&p.class_id).cloned().unwrap_or_default(),
                        ).await;
                        stat.0        += fetched;
                        stat.2        += errors;
                        total_fetched += fetched;
                        total_errors  += errors;
                        batch_new     += fetched;
                    }
                    batch_total_new     += batch_new;
                    batch_total_skipped += batch_skipped;
                    LogRepository::insert(app, pool, Some(session_id), "FETCH", "fetch",
                        &format!("{}: {} results → {} new, {} skipped", label, presets.len(), batch_new, batch_skipped),
                    ).await.ok();
                }
                Err(e) => {
                    stat.2 += 1;
                    total_errors += 1;
                    LogRepository::insert(app, pool, Some(session_id), "ERR", "fetch",
                        &format!("{} failed: {}", label, e)).await.ok();
                }
            }
        }

        LogRepository::insert(app, pool, Some(session_id), "SYNC", "fetch",
            &format!("Batch saved: {} new, {} skipped", batch_total_new, batch_total_skipped),
        ).await.ok();
    }

    // Per-class summaries + events
    for (i, class) in class_entries.iter().enumerate() {
        let (fetched, skipped, errors) = class_stats.get(&class.db_id).copied().unwrap_or((0, 0, 0));
        LogRepository::insert(app, pool, Some(session_id), "ORCH", "fetch", &format!(
            "{}: {} new | {} skipped (DB) | {} errors",
            class.name, fetched, skipped, errors,
        )).await.ok();

        if class.db_id >= 0 {
            SessionRepository::upsert_class_stats(
                pool, session_id, class.db_id, fetched as i32, 0, errors as i32,
            ).await.ok();
        }

        Events::class_stats_updated(app, ClassStatsUpdated {
            class_id: class.db_id.max(0) as u32,
            fetched,
            images:   0,
            errors,
            skipped,
        });

        Events::fetch_progress(app, FetchProgress {
            class_id:   class.db_id.max(0) as u32,
            class_name: class.name.clone(),
            fetched:    i + 1,
            total:      total_classes,
        });
    }

    Events::fetch_done(app);
    Ok(FetchResult { total_fetched, total_errors, total_skipped })
}

// ── Phase: Download pipeline ──────────────────────────────────

pub async fn run_download_pipeline(
    app:         &AppHandle,
    browser:     Arc<BrowserSession>,
    r2:          &R2Client,
    pool:        &PgPool,
    session_id:  i64,
    cancel:      Arc<AtomicBool>,
    img_done:    Arc<AtomicUsize>,
    img_total:   Arc<AtomicUsize>,
    mut rx:      mpsc::Receiver<ImageTask>,
    parallelism: usize,
) -> DownloadResult {
    let mut total_uploaded = 0usize;
    let mut total_errors   = 0usize;
    let mut total_done     = 0usize;

    let upload_done = Arc::new(AtomicUsize::new(0));

    let sem = Arc::new(Semaphore::new(parallelism));
    let mut set: JoinSet<(usize, usize, usize)> = JoinSet::new();

    while let Some(task) = rx.recv().await {
        while set.len() >= parallelism {
            if let Some(Ok((done, uploaded, errors))) = set.join_next().await {
                total_done     += done;
                total_uploaded += uploaded;
                total_errors   += errors;
            }
        }

        if cancel.load(Ordering::Relaxed) { continue; }

        let permit        = Arc::clone(&sem).acquire_owned().await.unwrap();
        let app_h         = app.clone();
        let pool_h        = pool.clone();
        let r2_h          = r2.clone();
        let browser_h     = Arc::clone(&browser);
        let cancel_h      = Arc::clone(&cancel);
        let img_done_h    = Arc::clone(&img_done);
        let img_total_h   = Arc::clone(&img_total);
        let upload_done_h = Arc::clone(&upload_done);

        set.spawn(async move {
            let _permit = permit;
            let mut uploaded = 0usize;
            let mut errors   = 0usize;

            Events::scrapper_progress(&app_h, ScrapperProgress {
                preset_id:     task.preset_id.to_string(),
                class_id:      task.class_id as u32,
                class_name:    task.class_name.clone(),
                current:       img_done_h.load(Ordering::Relaxed),
                total:         img_total_h.load(Ordering::Relaxed),
                status:        ProgressStatus::Processing,
                message:       format!("Downloading preset {}", task.preset_id),
                progress_type: ProgressType::Popular,
            });

            if cancel_h.load(Ordering::Relaxed) { return (1, 0, 0); }

            let images = browser_h.fetch_preset_images(
                task.preset_id,
                task.image_1.as_deref(),
                task.image_2.as_deref(),
            ).await;

            let (img1, img2) = match images {
                Ok(pair) => pair,
                Err(e) => {
                    // Navigation/timeout error — do NOT mark as not_found so the pending loop retries it next session
                    LogRepository::insert(&app_h, &pool_h, Some(session_id), "WARN", "img_fetch",
                        &format!("preset {} page failed (will retry): {}", task.preset_id, e)).await.ok();
                    img_done_h.fetch_add(1, Ordering::Relaxed);
                    return (1, 0, 1);
                }
            };

            let mut r2_url_1  = None::<String>; // full URL for events
            let mut r2_url_2  = None::<String>;
            let mut db_path_1 = None::<String>; // /images/... path for DB
            let mut db_path_2 = None::<String>;
            let mut nf1 = None;
            let mut nf2 = None;

            match img1 {
                Some((filename, bytes)) => {
                    let key = format!("images/{}/{}/{}", task.class_name, task.preset_id, filename);
                    match r2_h.upload(&key, bytes).await {
                        Ok(url) => {
                            let ud = upload_done_h.fetch_add(1, Ordering::Relaxed) + 1;
                            Events::upload_progress(&app_h, UploadProgress {
                                preset_id: task.preset_id.to_string(),
                                image_url: url.clone(),
                                done:  ud,
                                total: img_total_h.load(Ordering::Relaxed) * 2,
                            });
                            db_path_1 = Some(format!("/{}", key));
                            r2_url_1  = Some(url);
                            uploaded += 1;
                        }
                        Err(e) => {
                            errors += 1;
                            nf1 = Some("not_found");
                            LogRepository::insert(&app_h, &pool_h, Some(session_id), "ERR", "img_upload",
                                &format!("preset {} img1 upload failed: {}", task.preset_id, e)).await.ok();
                        }
                    }
                }
                None => {
                    nf1 = Some("not_found");
                    LogRepository::insert(&app_h, &pool_h, Some(session_id), "WARN", "img_fetch",
                        &format!("preset {} img1 not found on page", task.preset_id)).await.ok();
                }
            }

            match img2 {
                Some((filename, bytes)) => {
                    let key = format!("images/{}/{}/{}", task.class_name, task.preset_id, filename);
                    match r2_h.upload(&key, bytes).await {
                        Ok(url) => {
                            let ud = upload_done_h.fetch_add(1, Ordering::Relaxed) + 1;
                            Events::upload_progress(&app_h, UploadProgress {
                                preset_id: task.preset_id.to_string(),
                                image_url: url.clone(),
                                done:  ud,
                                total: img_total_h.load(Ordering::Relaxed) * 2,
                            });
                            db_path_2 = Some(format!("/{}", key));
                            r2_url_2  = Some(url);
                            uploaded += 1;
                        }
                        Err(e) => {
                            errors += 1;
                            nf2 = Some("not_found");
                            LogRepository::insert(&app_h, &pool_h, Some(session_id), "ERR", "img_upload",
                                &format!("preset {} img2 upload failed: {}", task.preset_id, e)).await.ok();
                        }
                    }
                }
                None => {
                    nf2 = Some("not_found");
                    LogRepository::insert(&app_h, &pool_h, Some(session_id), "WARN", "img_fetch",
                        &format!("preset {} img2 not found on page", task.preset_id)).await.ok();
                }
            }

            if db_path_1.is_some() || db_path_2.is_some() {
                let img_count = db_path_1.is_some() as u8 + db_path_2.is_some() as u8;
                LogRepository::insert(&app_h, &pool_h, Some(session_id), "SYNC", "img_upload",
                    &format!("preset {} — {} image(s) uploaded", task.preset_id, img_count)).await.ok();
                PresetRepository::update_image_urls(&pool_h, task.preset_id, db_path_1.as_deref(), db_path_2.as_deref()).await.ok();
            }
            if nf1.is_some() || nf2.is_some() {
                PresetRepository::update_image_names(&pool_h, task.preset_id, nf1, nf2).await.ok();
            }

            let done = img_done_h.fetch_add(1, Ordering::Relaxed) + 1;

            Events::image_progress(&app_h, ImageProgress {
                preset_id:  task.preset_id.to_string(),
                class_name: task.class_name.clone(),
                image_num:  1,
                done,
                total: img_total_h.load(Ordering::Relaxed),
            });

            if r2_url_1.is_some() || r2_url_2.is_some() {
                Events::preset_synced(&app_h, PresetSynced {
                    preset_id:      task.preset_id.to_string(),
                    class_id:       task.class_id as u32,
                    image_1_url:    r2_url_1,
                    image_2_url:    r2_url_2,
                    downloads:      Some(task.downloads),
                    views:          Some(task.views),
                    likes:          Some(task.likes),
                    character_name: task.character_name.clone(),
                });
            }

            (1, uploaded, errors)
        });

    }

    // Drain remaining in-flight tasks
    while let Some(Ok((done, uploaded, errors))) = set.join_next().await {
        total_done     += done;
        total_uploaded += uploaded;
        total_errors   += errors;
    }

    DownloadResult { total_images: total_done, total_uploaded, total_errors }
}

// ── Helpers ───────────────────────────────────────────────────

fn norm_image(s: Option<&str>) -> Option<&str> {
    match s {
        Some(v) if !v.is_empty() => Some(v),
        _ => Some("not_found"),
    }
}

fn is_real_image(s: Option<&str>) -> bool {
    matches!(s, Some(v) if !v.is_empty() && v != "not_found")
}

async fn insert_preset_and_queue(
    app:        &AppHandle,
    pool:       &PgPool,
    session_id: i64,
    tx:         &mpsc::Sender<ImageTask>,
    img_total:  &AtomicUsize,
    p:          &GarmothPreset,
    class_name: String,
) -> (usize, usize) {
    let img1 = norm_image(p.image_1.as_deref());
    let img2 = norm_image(p.image_2.as_deref());

    match PresetRepository::insert_new(
        pool, p.id, p.class_id,
        p.title.as_deref(), p.user_nickname.as_deref(), p.character_name.as_deref(),
        p.downloads, p.views, p.likes,
        img1, img2,
        p.creation_at, p.customizing_id, p.region.as_deref(), p.score,
    ).await {
        Ok(true) => {
            if is_real_image(img1) || is_real_image(img2) {
                img_total.fetch_add(1, Ordering::Relaxed);
                tx.send(ImageTask {
                    preset_id:      p.id,
                    class_id:       p.class_id,
                    class_name,
                    image_1:        img1.filter(|&s| s != "not_found").map(str::to_owned),
                    image_2:        img2.filter(|&s| s != "not_found").map(str::to_owned),
                    downloads:      p.downloads,
                    views:          p.views,
                    likes:          p.likes,
                    character_name: p.character_name.clone(),
                }).await.ok();
            }
            Events::preset_synced(app, PresetSynced {
                preset_id:      p.id.to_string(),
                class_id:       p.class_id as u32,
                image_1_url:    None,
                image_2_url:    None,
                downloads:      None,
                views:          None,
                likes:          None,
                character_name: None,
            });
            (1, 0)
        }
        Ok(false) => (0, 0),
        Err(e) => {
            LogRepository::insert(app, pool, Some(session_id), "ERR", "insert_new",
                &format!("preset {} failed: {}", p.id, e)).await.ok();
            (0, 1)
        }
    }
}


// ── Phase: Pending image loop ─────────────────────────────────
//
// Runs concurrently with run_fetch. Each round queries the top-10 pending presets
// per class (by downloads), enqueues them, then waits until that batch is processed
// before querying again. Repeats until no pending images remain.

async fn run_pending_loop(
    app:        &AppHandle,
    pool:       &PgPool,
    session_id: i64,
    cancel:     Arc<AtomicBool>,
    tx:         mpsc::Sender<ImageTask>,
    img_total:  Arc<AtomicUsize>,
) {
    use std::collections::HashSet;

    if cancel.load(Ordering::Relaxed) { return; }

    let class_name_map: std::collections::HashMap<i32, String> =
        ClassRepository::get_all(pool).await
            .unwrap_or_default()
            .into_iter()
            .map(|c| (c.id, c.display))
            .collect();

    // Track every ID ever sent this session to avoid double-queueing.
    let mut all_sent: HashSet<i64> = HashSet::new();
    let mut round = 0usize;

    loop {
        if cancel.load(Ordering::Relaxed) { return; }

        let batch: Vec<_> = PresetRepository::get_pending_downloads(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|(id, ..)| !all_sent.contains(id))
            .collect();

        if batch.is_empty() {
            if round > 0 {
                LogRepository::insert(app, pool, Some(session_id), "ORCH", "pending",
                    "No more pending images — pending loop done",
                ).await.ok();
            }
            return;
        }

        round += 1;
        let batch_ids: HashSet<i64> = batch.iter().map(|(id, ..)| *id).collect();
        all_sent.extend(&batch_ids);

        LogRepository::insert(app, pool, Some(session_id), "ORCH", "pending",
            &format!("Pending round {}: queuing {} preset(s) (top-10 per class by downloads)", round, batch.len()),
        ).await.ok();

        for (id, class_id, image_1, image_2, downloads, views, likes, character_name) in batch {
            if cancel.load(Ordering::Relaxed) { return; }
            img_total.fetch_add(1, Ordering::Relaxed);
            let class_name = class_name_map.get(&class_id).cloned().unwrap_or_default();
            if tx.send(ImageTask {
                preset_id: id,
                class_id,
                class_name,
                image_1,
                image_2,
                downloads,
                views,
                likes,
                character_name,
            }).await.is_err() { return; }
        }

        // Wait for this batch to finish: poll DB every 15s until none of the sent IDs
        // remain pending. 30-minute safety timeout handles stuck/errored presets.
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_secs(30 * 60);
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            if cancel.load(Ordering::Relaxed) { return; }

            let still_pending = PresetRepository::get_pending_downloads(pool)
                .await
                .unwrap_or_default();
            let any_in_batch = still_pending.iter().any(|(id, ..)| batch_ids.contains(id));

            if !any_in_batch || tokio::time::Instant::now() >= deadline { break; }
        }
        // Query next batch
    }
}

// ── Session lifecycle helpers ─────────────────────────────────

async fn abort_session(app: &AppHandle, pool: &PgPool, session_id: i64, e: AppError) {
    LogRepository::insert(app, pool, Some(session_id), "ERR", "session",
        &format!("Session failed: {}", e),
    ).await.ok();
    clear_session(app);
    SessionRepository::finish(pool, session_id, "error", 0, 0, 0, 1, 0, 0).await.ok();
    Events::scrapper_error(app, ScrapperError { message: e.to_string(), phase: ScrapperPhase::Fetch });
}

async fn finish_session(
    app:        &AppHandle,
    pool:       &PgPool,
    session_id: i64,
    status:     &str,
    fetched:    usize,
    images:     usize,
    uploaded:   usize,
    errors:     usize,
    skipped:    usize,
    started:    Instant,
) {
    clear_session(app);
    SessionRepository::finish(
        pool, session_id, status,
        fetched  as i32,
        images   as i32,
        uploaded as i32,
        errors   as i32,
        skipped  as i32,
        started.elapsed().as_secs() as i32,
    ).await.ok();
}

fn clear_session(app: &AppHandle) {
    let state: tauri::State<'_, AppState> = app.state();
    *state.current_session.lock().unwrap() = None;
}
