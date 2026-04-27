use headless_chrome::{Browser, LaunchOptions};
use std::sync::{Mutex, OnceLock};

const HEADLESS_CHROME_WINDOW_SIZE: u32 = 2000;

static DRAWIO_BROWSER: OnceLock<Mutex<Option<Browser>>> = OnceLock::new();

pub(super) struct DrawioBrowserOps;

impl DrawioBrowserOps {
    pub(super) fn open_tab(
        temp_html: &tempfile::NamedTempFile,
    ) -> Result<std::sync::Arc<headless_chrome::Tab>, anyhow::Error> {
        let browser = Self::shared_browser()?;
        match Self::open_tab_with_browser(&browser, temp_html) {
            Ok(tab) => Ok(tab),
            Err(first_error) => {
                Self::reset_browser();
                let browser = Self::shared_browser()?;
                Self::open_tab_with_browser(&browser, temp_html).map_err(|second_error| {
                    anyhow::anyhow!(
                        "Failed to open Draw.io tab: {first_error}; retry failed: {second_error}"
                    )
                })
            }
        }
    }

    fn shared_browser() -> Result<Browser, anyhow::Error> {
        let mut lock = Self::browser_slot()
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock Draw.io browser: {e}"))?;
        if let Some(browser) = lock.as_ref() {
            return Ok(browser.clone());
        }

        let browser = Self::launch_browser()?;
        *lock = Some(browser.clone());
        Ok(browser)
    }

    pub(super) fn reset_browser() {
        let Ok(mut lock) = Self::browser_slot().lock() else {
            return;
        };
        *lock = None;
    }

    fn browser_slot() -> &'static Mutex<Option<Browser>> {
        DRAWIO_BROWSER.get_or_init(|| Mutex::new(None))
    }

    fn launch_browser() -> Result<Browser, anyhow::Error> {
        let launch_options = LaunchOptions::default_builder()
            .window_size(Some((
                HEADLESS_CHROME_WINDOW_SIZE,
                HEADLESS_CHROME_WINDOW_SIZE,
            )))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {e}"))?;

        Browser::new(launch_options).map_err(|e| anyhow::anyhow!("Failed to launch browser: {e}"))
    }

    fn open_tab_with_browser(
        browser: &Browser,
        temp_html: &tempfile::NamedTempFile,
    ) -> Result<std::sync::Arc<headless_chrome::Tab>, anyhow::Error> {
        let tab = browser
            .new_tab()
            .map_err(|e| anyhow::anyhow!("Failed to create tab: {e}"))?;
        let url = format!("file://{}", temp_html.path().display());
        tab.navigate_to(&url)
            .map_err(|e| anyhow::anyhow!("Navigation failed: {e}"))?;
        tab.wait_until_navigated()
            .map_err(|e| anyhow::anyhow!("Wait navigation failed: {e}"))?;
        Ok(tab)
    }
}
