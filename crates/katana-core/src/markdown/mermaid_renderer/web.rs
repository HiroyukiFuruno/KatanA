use crate::markdown::color_preset::DiagramColorPreset;
use std::path::Path;
use std::time::{Duration, Instant};

const MERMAID_RENDER_TIMEOUT_SECS: u64 = 15;

pub(super) struct MermaidWebRendererOps;

impl MermaidWebRendererOps {
    pub(super) fn render_with_headless_chrome(
        source: &str,
        mermaid_js: &Path,
        preset: &DiagramColorPreset,
    ) -> Result<Vec<u8>, String> {
        let encoded_source = serde_json::to_string(source).map_err(|e| e.to_string())?;
        let html = super::html::MermaidHtmlOps::build(&encoded_source, mermaid_js, preset);
        let temp_html = write_temp_html(html)?;
        let tab = super::browser::MermaidBrowserOps::open_tab(&temp_html)?;
        let result = wait_for_svg(&tab);
        let _ = tab.close(false);
        result
    }
}

fn write_temp_html(html: String) -> Result<tempfile::NamedTempFile, String> {
    let temp_html = tempfile::Builder::new()
        .prefix("katana_mermaid_")
        .suffix(".html")
        .tempfile()
        .map_err(|e| format!("Failed to create temp html file: {e}"))?;
    std::fs::write(temp_html.path(), html)
        .map_err(|e| format!("Failed to write temp html file: {e}"))?;
    Ok(temp_html)
}

fn wait_for_svg(tab: &headless_chrome::Tab) -> Result<Vec<u8>, String> {
    let start = Instant::now();
    loop {
        if read_render_state(tab).as_deref() == Some("true") {
            return extract_svg(tab);
        }

        if let Some(message) = read_render_error(tab) {
            return Err(message);
        }

        if start.elapsed() > Duration::from_secs(MERMAID_RENDER_TIMEOUT_SECS) {
            return Err("Mermaid rendering timed out".to_string());
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn extract_svg(tab: &headless_chrome::Tab) -> Result<Vec<u8>, String> {
    let element = tab
        .find_element("#diagram svg")
        .map_err(|e| format!("Failed to find Mermaid SVG element: {e}"))?;
    element
        .capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
        )
        .map_err(|e| format!("Screenshot failed: {e}"))
}

fn read_render_error(tab: &headless_chrome::Tab) -> Option<String> {
    if read_render_state(tab).as_deref() != Some("error") {
        return None;
    }

    Some(
        tab.evaluate("window.katanaRenderError", false)
            .ok()
            .and_then(|it| it.value)
            .and_then(|it| it.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "unknown Mermaid render error".to_string()),
    )
}

fn read_render_state(tab: &headless_chrome::Tab) -> Option<String> {
    tab.evaluate("document.body.getAttribute('data-katana-rendered')", false)
        .ok()
        .and_then(|it| it.value)
        .and_then(|it| it.as_str().map(ToOwned::to_owned))
}
