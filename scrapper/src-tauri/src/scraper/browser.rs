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

        // Also warm up the assets CDN so CF sets clearance for image downloads
        let page2 = context
            .new_page()
            .await
            .map_err(|e| AppError::Scrape(format!("warmup2 page: {:?}", e)))?;

        page2.goto(
            "https://assets.garmoth.com/",
            Some(GotoOptions {
                wait_until: Some(WaitUntil::Load),
                timeout:    Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        )
        .await
        .ok(); // non-fatal — CDN root may 404

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

    pub async fn download(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let page = self
            .context
            .new_page()
            .await
            .map_err(|e| AppError::Scrape(format!("download page: {:?}", e)))?;

        let response = page
            .goto(url, None)
            .await
            .map_err(|e| AppError::Scrape(format!("download goto: {:?}", e)))?;

        match response {
            Some(resp) => {
                if resp.status() == 403 {
                    return Err(AppError::CfBlocked);
                }
                resp.body()
                    .await
                    .map_err(|e| AppError::Scrape(format!("download body: {:?}", e)))
            }
            None => Err(AppError::Scrape(format!("no response for {}", url))),
        }
    }
}
