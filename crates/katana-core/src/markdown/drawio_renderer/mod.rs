pub mod types;

pub use types::*;

pub type DrawioRenderOps = DrawioRendererOps;

use crate::markdown::{DiagramBlock, DiagramResult};
use headless_chrome::{Browser, LaunchOptions};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

const DRAWIO_DOWNLOAD_URL: &str = "https://viewer.diagrams.net/js/viewer-static.min.js";
const HEADLESS_CHROME_WINDOW_SIZE: u32 = 2000;
const HEADLESS_CHROME_TIMEOUT_SECS: u64 = 15;

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
        block.source.hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = std::env::temp_dir().join("katana_drawio_cache");
        let _ = fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("{:016x}.png", hash));

        if let Ok(data) = fs::read(&cache_file) {
            return DiagramResult::OkPng(data);
        }

        match Self::render_with_headless_chrome(&block.source, &drawio_js) {
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

    fn render_with_headless_chrome(xml: &str, drawio_js: &Path) -> Result<Vec<u8>, anyhow::Error> {
        let escaped_json_string = serde_json::to_string(xml)?;
        let drawio_url = format!("file://{}", drawio_js.to_string_lossy());

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <script src="{drawio_url}"></script>
</head>
<body>
  <div id="graph-container" class="mxgraph"></div>
  <script>
    window.onload = function() {{
      var container = document.getElementById('graph-container');
      container.setAttribute('data-mxgraph', JSON.stringify({{ xml: {} }}));
      
      if (typeof GraphViewer !== 'undefined') {{
        GraphViewer.processElements();
      }} else {{
        document.body.innerHTML += '<div class=\"error\">GraphViewer undefined</div>';
      }}
    }};
  </script>
</body>
</html>"#,
            escaped_json_string,
            drawio_url = drawio_url
        );

        let temp_html = tempfile::Builder::new()
            .prefix("katana_drawio_")
            .suffix(".html")
            .tempfile()?;

        std::fs::write(temp_html.path(), &html)?;

        let launch_options = LaunchOptions::default_builder()
            .window_size(Some((
                HEADLESS_CHROME_WINDOW_SIZE,
                HEADLESS_CHROME_WINDOW_SIZE,
            )))
            .user_data_dir(Some(
                std::env::temp_dir().join("katana_drawio_chrome_profile"),
            ))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?;

        let tab = browser
            .new_tab()
            .map_err(|e| anyhow::anyhow!("Failed to create tab: {}", e))?;

        let url = format!("file://{}", temp_html.path().display());
        tab.navigate_to(&url)
            .map_err(|e| anyhow::anyhow!("Navigation failed: {}", e))?;
        tab.wait_until_navigated()
            .map_err(|e| anyhow::anyhow!("Wait navigation failed: {}", e))?;

        tab.wait_for_element_with_custom_timeout(
            ".mxgraph svg",
            Duration::from_secs(HEADLESS_CHROME_TIMEOUT_SECS),
        )
        .map_err(|e| {
            let html = tab
                .get_content()
                .unwrap_or_else(|_| "Failed to get content".to_string());
            anyhow::anyhow!("SVG rendering timeout: {}. HTML: {}", e, html)
        })?;

        let element = tab
            .find_element(".mxgraph svg")
            .map_err(|e| anyhow::anyhow!("Failed to find SVG element: {}", e))?;

        element
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            )
            .map_err(|e| anyhow::anyhow!("Screenshot failed: {}", e))
    }
}

#[cfg(test)]
mod tests;
