use super::native_document::NativeHtmlDocument;
use super::types::ImageExporter;
use crate::markdown::MarkdownError;

impl ImageExporter {
    pub fn is_available() -> bool {
        true
    }

    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        let document = NativeHtmlDocument::parse(html)?;
        let image = document.render_image()?;
        if Self::is_jpeg(output) {
            image.save_jpeg(output)
        } else {
            image.save_png(output)
        }
    }

    fn is_jpeg(output: &std::path::Path) -> bool {
        matches!(
            output
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("jpg" | "jpeg")
        )
    }
}
