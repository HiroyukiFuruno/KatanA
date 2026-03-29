use crate::markdown::MarkdownError;

/// Default fallback viewport width in pixels for screenshot captures.
const DEFAULT_VIEWPORT_WIDTH: f64 = 1280.0;
/// Default fallback viewport height in pixels for screenshot captures.
const DEFAULT_VIEWPORT_HEIGHT: f64 = 800.0;
/// Scale factor for retina-quality screenshot captures.
const SCREENSHOT_SCALE: f64 = 2.0;

/// Exporter for generating image files (PNG/JPG) via Headless Chrome.
pub struct ImageExporter;

impl ImageExporter {
    /// Returns true if Headless Chrome can be initialized.
    pub fn is_available() -> bool {
        true
    }

    /// Exports the given HTML content to an image file at the specified path.
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use headless_chrome::{
            protocol::cdp::{Emulation, Page},
            Browser, LaunchOptions,
        };
        use std::io::Write;

        let options = LaunchOptions::default();
        let browser =
            Browser::new(options).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        let tab = browser
            .new_tab()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        // WHY: Set viewport to a reasonable width before navigation so content
        // WHY: is laid out correctly (not in a tiny default viewport).
        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: DEFAULT_VIEWPORT_WIDTH as u32,
            height: DEFAULT_VIEWPORT_HEIGHT as u32,
            device_scale_factor: SCREENSHOT_SCALE,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        let mut temp = tempfile::Builder::new()
            .prefix("katana_export_src_")
            .suffix(".html")
            .tempfile()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        temp.write_all(html.as_bytes())
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        let url = format!("file://{}", temp.path().display());

        tab.navigate_to(&url)
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        let format = if output
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase() == "jpg" || s.to_lowercase() == "jpeg")
            .unwrap_or(false)
        {
            Page::CaptureScreenshotFormatOption::Jpeg
        } else {
            Page::CaptureScreenshotFormatOption::Png
        };

        // WHY: Get full document dimensions via JavaScript
        let dimensions = tab
            .evaluate(
                "JSON.stringify({width: Math.max(document.body.scrollWidth, document.documentElement.scrollWidth), height: Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)})",
                false,
            )
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        // WHY: Resize viewport to match the actual document height so the full
        // WHY: page is in-view, then capture without clip (captures the viewport).
        if let Some(serde_json::Value::String(json_str)) = dimensions.value {
            if let Some(dims) = serde_json::from_str::<serde_json::Value>(&json_str).ok() {
                let h = dims["height"].as_f64().unwrap_or(DEFAULT_VIEWPORT_HEIGHT) as u32;
                let _ = tab.call_method(Emulation::SetDeviceMetricsOverride {
                    width: DEFAULT_VIEWPORT_WIDTH as u32,
                    height: h,
                    device_scale_factor: SCREENSHOT_SCALE,
                    mobile: false,
                    scale: None,
                    screen_width: None,
                    screen_height: None,
                    position_x: None,
                    position_y: None,
                    dont_set_visible_size: None,
                    screen_orientation: None,
                    viewport: None,
                    display_feature: None,
                    device_posture: None,
                });
            }
        }

        // WHY: Capture screenshot of the full viewport (which now matches the document height)
        let img_data = tab
            .capture_screenshot(format, None, None, true)
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        std::fs::write(output, img_data).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        Ok(())
    }
}
