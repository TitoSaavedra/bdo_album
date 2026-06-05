use std::time::Duration;

use playwright_rs::{
    Browser, BrowserContext, BrowserContextOptions, GotoOptions, LaunchOptions,
    Playwright, WaitUntil, install_browsers,
};

use crate::core::errors::AppError;

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
    pub async fn new() -> Result<Self, AppError> {
        #[cfg(all(target_os = "windows", not(debug_assertions)))]
        hidden_console::ensure_hidden();

        install_browsers(Some(&["chromium"]))
            .await
            .map_err(|e| AppError::Scrape(format!("browser install: {:?}", e)))?;

        let playwright = Playwright::launch()
            .await
            .map_err(|e| AppError::Scrape(format!("playwright launch: {:?}", e)))?;

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

        // Warm-up: establish CF session on garmoth.com
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
            page.route("**/beauty-album/images/**", move |route| {
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
            }).await.map_err(|e| AppError::Scrape(format!("route preset {}: {:?}", preset_id, e)))?;
        }

        let preset_url = format!("https://garmoth.com/beauty-album/preset/{}", preset_id);
        page.goto(
            &preset_url,
            Some(GotoOptions {
                wait_until: Some(WaitUntil::DomContentLoaded),
                timeout:    Some(Duration::from_secs(60)),
                ..Default::default()
            }),
        ).await.map_err(|e| AppError::Scrape(format!("goto preset {}: {:?}", preset_id, e)))?;

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

        Ok((img1, img2))
    }

    async fn download_image(&self, pair: Option<(String, String)>) -> Option<(String, Vec<u8>)> {
        let (name, url) = pair?;
        let page = self.context.new_page().await.ok()?;
        let resp = match page.goto(&url, None).await {
            Ok(Some(r)) => r,
            _ => return None,
        };
        if resp.status() >= 400 { return None; }
        let bytes = resp.body().await.ok()?;
        if bytes.is_empty() { None } else { Some((name, bytes)) }
    }

}
