use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{render, DiagramRenderer, MarkdownError};

/// Exporter for generating standalone HTML documents.
pub struct HtmlExporter;

impl HtmlExporter {
    /// Exports Markdown as a standalone HTML document with embedded CSS matching the given preset.
    ///
    /// When `base_dir` is provided, relative image paths in the rendered HTML are
    /// resolved to absolute `file://` URLs so that images display correctly even
    /// when the HTML is opened from a different directory (e.g. a temp file).
    pub fn export<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        let output = render(source, renderer)?;

        let bg_color = Self::get_bg_color(preset);

        let props = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji'";
        let monos = "SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace";

        let css = Self::generate_css(preset, bg_color, props, monos);

        let body = if let Some(dir) = base_dir {
            Self::resolve_relative_paths(&output.html, dir)
        } else {
            output.html
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Exported Document</title>
<style>
{}
</style>
</head>
<body>
{}
</body>
</html>"#,
            css, body
        );

        Ok(html)
    }

    /// Resolves relative `src` attributes in `<img>` tags to absolute `file://` URLs.
    fn resolve_relative_paths(html: &str, base_dir: &std::path::Path) -> String {
        // Match src="..." that don't start with http://, https://, data:, or file://
        let re = regex::Regex::new(r#"src="([^"]+)""#).unwrap();
        re.replace_all(html, |caps: &regex::Captures| {
            let src = &caps[1];
            if src.starts_with("http://")
                || src.starts_with("https://")
                || src.starts_with("data:")
                || src.starts_with("file://")
            {
                caps[0].to_string()
            } else {
                let abs = base_dir.join(src);
                format!("src=\"file://{}\"", abs.display())
            }
        })
        .to_string()
    }

    fn get_bg_color(preset: &DiagramColorPreset) -> &str {
        if preset.background == "transparent" {
            if preset.text == "#E0E0E0" {
                "#1e1e1e"
            } else {
                "#ffffff"
            }
        } else {
            preset.background
        }
    }

    fn generate_css(
        preset: &DiagramColorPreset,
        bg_color: &str,
        props: &str,
        monos: &str,
    ) -> String {
        format!(
            r#"
body {{
    font-family: {props};
    background-color: {bg_color};
    color: {text};
    line-height: 1.6;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
}}
h1, h2, h3, h4, h5, h6 {{
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
}}
h1 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
h2 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
a {{ color: #0366d6; text-decoration: none; }}
pre {{
    background-color: {fill};
    border: 1px solid {stroke};
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
    line-height: 1.5;
}}
code {{
    font-family: {monos};
    background-color: {fill};
    border-radius: 3px;
    padding: 0.2em 0.4em;
    font-size: 85%;
}}
pre code {{ background-color: transparent; padding: 0; }}
blockquote {{ border-left: 0.25em solid {stroke}; color: {text}; opacity: 0.8; padding: 0 1em; margin: 0; }}
table {{ border-spacing: 0; border-collapse: collapse; margin-top: 0; margin-bottom: 16px; }}
table th, table td {{ padding: 6px 13px; border: 1px solid {stroke}; }}
img {{ max-width: 100%; box-sizing: content-box; background-color: {bg_color}; }}
.katana-diagram img {{ background-color: transparent; }}
hr {{ height: 0.25em; padding: 0; margin: 24px 0; background-color: {stroke}; border: 0; }}
            "#,
            props = props,
            bg_color = bg_color,
            text = preset.text,
            stroke = preset.stroke,
            fill = preset.fill,
            monos = monos,
        )
    }
}

/// Exporter for generating PDF documents via Headless Chrome.
pub struct PdfExporter;

impl PdfExporter {
    /// Returns true if Headless Chrome can be initialized.
    pub fn is_available() -> bool {
        // We assume it's available as headless_chrome can download a browser.
        true
    }

    /// Exports the given HTML content to a PDF file at the specified path.
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use headless_chrome::{Browser, LaunchOptions};
        use std::io::Write;

        let options = LaunchOptions::default();
        let browser =
            Browser::new(options).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        let tab = browser
            .new_tab()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        // Use a temporary file to avoid URL length limits with data: URIs.
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

        let pdf_options = headless_chrome::types::PrintToPdfOptions {
            print_background: Some(true),
            ..Default::default()
        };

        let pdf_data = tab
            .print_to_pdf(Some(pdf_options))
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        std::fs::write(output, pdf_data).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        Ok(())
    }
}

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

        // Set viewport to a reasonable width before navigation so content
        // is laid out correctly (not in a tiny default viewport).
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

        // Get full document dimensions via JavaScript
        let dimensions = tab
            .evaluate(
                "JSON.stringify({width: Math.max(document.body.scrollWidth, document.documentElement.scrollWidth), height: Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)})",
                false,
            )
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        // Resize viewport to match the actual document height so the full
        // page is in-view, then capture without clip (captures the viewport).
        if let Some(serde_json::Value::String(json_str)) = dimensions.value {
            if let Ok(dims) = serde_json::from_str::<serde_json::Value>(&json_str) {
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

        // Capture screenshot of the full viewport (which now matches the document height)
        let img_data = tab
            .capture_screenshot(format, None, None, true)
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        std::fs::write(output, img_data).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        Ok(())
    }
}
