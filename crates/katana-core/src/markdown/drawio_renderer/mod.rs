mod browser;
mod html;
pub mod types;

pub use types::*;

pub type DrawioRenderOps = DrawioRendererOps;

use crate::markdown::{DiagramBlock, DiagramResult};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DRAWIO_DOWNLOAD_URL: &str = "https://viewer.diagrams.net/js/viewer-static.min.js";
const HEADLESS_CHROME_TIMEOUT_SECS: u64 = 15;
const DRAWIO_RENDER_POLL_INTERVAL_MS: u64 = 10;

impl DrawioRendererOps {
    pub fn default_install_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".local").join("katana").join("drawio.min.js"))
    }

    pub fn resolve_drawio_js() -> PathBuf {
        #[allow(clippy::single_match)]
        match std::env::var("DRAWIO_JS") {
            Ok(path) => return PathBuf::from(path),
            Err(_) => {}
        }

        Self::default_install_path().unwrap_or_else(|| PathBuf::from("drawio.min.js"))
    }

    pub fn find_drawio_js() -> Option<PathBuf> {
        let path = Self::resolve_drawio_js();
        path.exists().then_some(path)
    }

    pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
        let Some(drawio_js) = Self::find_drawio_js() else {
            return DiagramResult::NotInstalled {
                kind: "Draw.io".to_string(),
                download_url: DRAWIO_DOWNLOAD_URL.to_string(),
                install_path: Self::resolve_drawio_js(),
            };
        };

        let mut hasher = DefaultHasher::new();
        "drawio-png-v3".hash(&mut hasher);
        block.source.hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = std::env::temp_dir().join("katana_drawio_cache");
        let _ = fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("{:016x}.png", hash));

        if let Some(data) = Self::read_cached_png(&cache_file) {
            return DiagramResult::OkPng(data);
        }

        match Self::render_with_retry(&block.source, &drawio_js) {
            Ok(data) => {
                let _ = fs::write(&cache_file, &data);
                DiagramResult::OkPng(data)
            }
            Err(e) => {
                tracing::warn!("Headless Chrome Draw.io rendering failed: {:?}", e);
                DiagramResult::Err {
                    source: block.source.clone(),
                    error: format!("Headless Chrome rendering failed: {}", e),
                }
            }
        }
    }

    fn render_with_retry(xml: &str, drawio_js: &Path) -> Result<Vec<u8>, anyhow::Error> {
        match Self::render_with_headless_chrome(xml, drawio_js) {
            Ok(data) => Ok(data),
            Err(first_error) => {
                browser::DrawioBrowserOps::reset_browser();
                Self::render_with_headless_chrome(xml, drawio_js).map_err(|second_error| {
                    anyhow::anyhow!(
                        "Draw.io rendering failed: {first_error}; retry failed: {second_error}"
                    )
                })
            }
        }
    }

    fn render_with_headless_chrome(xml: &str, drawio_js: &Path) -> Result<Vec<u8>, anyhow::Error> {
        let temp_html = html::DrawioHtmlOps::write_temp_html(xml, drawio_js)?;

        let tab = browser::DrawioBrowserOps::open_tab(&temp_html)?;

        Self::process_graph_elements(&tab)?;
        Self::wait_for_svg(&tab)?;
        Self::wait_for_next_frame(&tab)?;
        let data = Self::capture_rendered_graph(&tab);
        let _ = tab.close(false);
        data
    }

    fn process_graph_elements(tab: &headless_chrome::Tab) -> Result<(), anyhow::Error> {
        let start = Instant::now();
        loop {
            match Self::try_process_graph_elements(tab)? {
                DrawioProcessState::Ready => return Ok(()),
                DrawioProcessState::Pending => {
                    if start.elapsed() > Duration::from_secs(HEADLESS_CHROME_TIMEOUT_SECS) {
                        return Err(anyhow::anyhow!("GraphViewer was not loaded"));
                    }
                    std::thread::sleep(Duration::from_millis(DRAWIO_RENDER_POLL_INTERVAL_MS));
                }
            }
        }
    }

    fn try_process_graph_elements(
        tab: &headless_chrome::Tab,
    ) -> Result<DrawioProcessState, anyhow::Error> {
        let result = tab
            .evaluate(
                r#"
                (() => {
                    if (typeof GraphViewer === 'undefined') {
                        return 'pending';
                    }
                    GraphViewer.processElements();
                    return 'ready';
                })()
                "#,
                false,
            )
            .map_err(|e| anyhow::anyhow!("Failed to process Draw.io graph elements: {e}"))?;

        match result
            .value
            .and_then(|it| it.as_str().map(ToOwned::to_owned))
        {
            Some(value) if value == "ready" => Ok(DrawioProcessState::Ready),
            _ => Ok(DrawioProcessState::Pending),
        }
    }

    fn wait_for_svg(tab: &headless_chrome::Tab) -> Result<(), anyhow::Error> {
        tab.wait_for_element_with_custom_timeout(
            "#graph-container svg",
            Duration::from_secs(HEADLESS_CHROME_TIMEOUT_SECS),
        )
        .map(|_| ())
        .map_err(|e| {
            let html = tab
                .get_content()
                .unwrap_or_else(|_| "Failed to get content".to_string());
            anyhow::anyhow!("SVG rendering timeout: {e}. HTML: {html}")
        })
    }

    fn wait_for_next_frame(tab: &headless_chrome::Tab) -> Result<(), anyhow::Error> {
        tab.evaluate(
            "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
            true,
        )
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("Failed to wait Draw.io paint frame: {e}"))
    }

    fn capture_rendered_graph(tab: &headless_chrome::Tab) -> Result<Vec<u8>, anyhow::Error> {
        let element = tab
            .find_element("#graph-container")
            .map_err(|e| anyhow::anyhow!("Failed to find Draw.io graph element: {e}"))?;
        element
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            )
            .map_err(|e| anyhow::anyhow!("Screenshot failed: {e}"))
    }

    fn read_cached_png(cache_file: &Path) -> Option<Vec<u8>> {
        fs::read(cache_file).ok()
    }
}

enum DrawioProcessState {
    Ready,
    Pending,
}

#[cfg(test)]
mod tests;
