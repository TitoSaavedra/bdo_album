use std::time::Duration;

use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, GotoOptions, LaunchOptions,
    Playwright, WaitUntil, install_browsers,
};

use sqlx::PgPool;
use tauri::AppHandle;

use crate::core::errors::AppError;
use crate::db::repositories::log_repo::LogRepository;

#[cfg(all(target_os = "windows", not(debug_assertions)))]
mod hidden_console {
    use std::sync::OnceLock;
    static DONE: OnceLock<()> = OnceLock::new();

    #[link(name = "kernel32")]
    extern "system" { fn AllocConsole() -> i32; fn GetConsoleWindow() -> isize; }
    #[link(name = "user32")]
    extern "system" { fn ShowWindow(hwnd: isize, nCmdShow: i32) -> i32; }

    pub fn ensure_hidden() {
        DONE.get_or_init(|| unsafe {
            if AllocConsole() != 0 {
                let hwnd = GetConsoleWindow();
                if hwnd != 0 { ShowWindow(hwnd, 0); }
            }
        });
    }
}

pub struct BrowserSession {
    _playwright: Playwright,
    _browser:    Browser,
    context:     BrowserContext,
}

impl BrowserSession {
    pub async fn new(app: &AppHandle, pool: &PgPool, session_id: i64) -> Result<Self, AppError> {
        #[cfg(all(target_os = "windows", not(debug_assertions)))]
        hidden_console::ensure_hidden();

        macro_rules! log {
            ($tag:expr, $msg:expr) => {
                LogRepository::insert(app, pool, Some(session_id), $tag, "browser", $msg).await.ok();
            };
        }

        log!("INFO", "Verifying Chromium installation");
        install_browsers(Some(&["chromium"]))
            .await
            .map_err(|e| AppError::Scrape(format!("browser install: {:?}", e)))?;

        log!("INFO", "Starting Playwright runtime");
        let playwright = Playwright::launch()
            .await
            .map_err(|e| AppError::Scrape(format!("playwright launch: {:?}", e)))?;

        log!("INFO", "Launching Chromium (headless)");
        let browser = playwright
            .chromium()
            .launch_with_options(
                LaunchOptions::default()
                    .headless(false)
                    .args(vec![
                        "--headless=new".to_string(),
                        "--disable-blink-features=AutomationControlled".to_string(),
                    ]),
            )
            .await
            .map_err(|e| AppError::Scrape(format!("browser launch: {:?}", e)))?;

        let context = browser
            .new_context_with_options(BrowserContextOptions {
                user_agent: Some(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                     (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
                        .to_string(),
                ),
                ..Default::default()
            })
            .await
            .map_err(|e| AppError::Scrape(format!("browser context: {:?}", e)))?;

        log!("INFO", "Navigating to garmoth.com (CF challenge)");
        let page = context
            .new_page()
            .await
            .map_err(|e| AppError::Scrape(format!("warmup page: {:?}", e)))?;

        page.goto(
            "https://garmoth.com/",
            Some(GotoOptions {
                wait_until: Some(WaitUntil::Load),
                timeout:    Some(Duration::from_secs(90)),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| AppError::Scrape(format!("warmup navigate: {:?}", e)))?;

        log!("INFO", "garmoth.com loaded — waiting for CF cookie");

        Ok(Self { _playwright: playwright, _browser: browser, context })
    }

    /// Polls for the cf_clearance cookie up to `timeout_secs` seconds.
    /// CF sets this cookie after the JS challenge completes — may take a few seconds.
    pub async fn wait_for_cf_clearance(&self, timeout_secs: u64) -> Option<String> {
        let deadline = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(timeout_secs);

        loop {
            if let Ok(cookies) = self.context
                .cookies(Some(&["https://garmoth.com"]))
                .await
            {
                if let Some(c) = cookies.into_iter().find(|c| c.name == "cf_clearance") {
                    return Some(c.value);
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    /// Opens the Garmoth preset page, captures image URLs via route interception,
    /// then downloads each image through the browser's Chromium network stack.
    /// Using route.continue_() avoids route.fetch() which goes through APIRequestContext
    /// (a separate HTTP client that CF may reject with a challenge page instead of the image).
    pub async fn fetch_preset_images(
        &self,
        preset_id: i64,
        image_1:   Option<&str>,
        image_2:   Option<&str>,
    ) -> Result<(Option<(String, Vec<u8>)>, Option<(String, Vec<u8>)>), AppError> {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};

        let expected: std::collections::HashSet<String> = [image_1, image_2]
            .iter()
            .filter_map(|s| s.map(String::from))
            .collect();

        // Maps filename → full CDN URL, populated by route interception
        let captured: Arc<Mutex<HashMap<String, String>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let page = self.context.new_page().await
            .map_err(|e| AppError::Scrape(format!("page: {:?}", e)))?;

        // Intercept before navigation — record the full URL, then let browser load normally
        {
            let cap = Arc::clone(&captured);
            let exp = expected.clone();
            if let Err(e) = page.route("**/beauty-album/images/**", move |route| {
                let cap = Arc::clone(&cap);
                let exp = exp.clone();
                async move {
                    let url      = route.request().url().to_string();
                    let filename = url.rsplit('/').next().unwrap_or("").to_string();
                    if exp.contains(&filename) {
                        cap.lock().unwrap().entry(filename).or_insert(url);
                    }
                    route.continue_(None).await?;
                    Ok(())
                }
            }).await {
                let _ = page.close().await;
                return Err(AppError::Scrape(format!("route preset {}: {:?}", preset_id, e)));
            }
        }

        let preset_url = format!("https://garmoth.com/beauty-album/preset/{}", preset_id);
        if let Err(e) = page.goto(
            &preset_url,
            Some(GotoOptions {
                wait_until: Some(WaitUntil::DomContentLoaded),
                timeout:    Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        ).await {
            let _ = page.close().await;
            return Err(AppError::Scrape(format!("goto preset {}: {:?}", preset_id, e)));
        }

        // Wait up to 5s for both image URLs to be captured
        for _ in 0..5 {
            {
                let data = captured.lock().unwrap();
                let got1 = image_1.map_or(true, |n| data.contains_key(n));
                let got2 = image_2.map_or(true, |n| data.contains_key(n));
                if got1 && got2 { break; }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        let urls = captured.lock().unwrap().clone();

        let get_url = |name: Option<&str>| -> Option<(String, String)> {
            let n = name?;
            urls.get(n).map(|u| (n.to_string(), u.clone()))
        };

        let img1 = self.download_image(get_url(image_1)).await;
        let img2 = self.download_image(get_url(image_2)).await;

        let _ = page.close().await;

        Ok((img1, img2))
    }

    async fn download_image(&self, pair: Option<(String, String)>) -> Option<(String, Vec<u8>)> {
        let (name, url) = pair?;
        let page = self.context.new_page().await.ok()?;
        let resp = match page.goto(&url, Some(GotoOptions {
            timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        })).await {
            Ok(Some(r)) => r,
            _ => { let _ = page.close().await; return None; }
        };
        if resp.status() >= 400 {
            let _ = page.close().await;
            return None;
        }
        let bytes = resp.body().await.ok();
        let _ = page.close().await;
        match bytes {
            Some(b) if !b.is_empty() => Some((name, b)),
            _ => None,
        }
    }

}
