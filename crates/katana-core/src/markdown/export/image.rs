use super::native_document::NativeHtmlDocument;
use super::types::{
    ExportError, ExportFormat, ExportInput, ExportOutput, ExporterTrait, ImageExporter,
};
use crate::markdown::MarkdownError;

impl ImageExporter {
    pub fn is_available() -> bool {
        true
    }

    fn export_file(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        let document = NativeHtmlDocument::parse(html)?;
        let image = document.render_image()?;
        if Self::is_jpeg(output) {
            image.save_jpeg(output)
        } else {
            image.save_png(output)
        }
    }

    pub(super) fn is_jpeg(output: &std::path::Path) -> bool {
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

static IMAGE_FORMATS: &[ExportFormat] = &[ExportFormat::Png, ExportFormat::Jpeg];

impl ExporterTrait for ImageExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        let format = if Self::is_jpeg(&input.output_path) {
            ExportFormat::Jpeg
        } else {
            ExportFormat::Png
        };
        Self::export_file(&input.html_source, &input.output_path)
            .map(|()| ExportOutput {
                output_path: input.output_path.clone(),
                format,
            })
            .map_err(|e| ExportError::RenderFailed(e.to_string()))
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        IMAGE_FORMATS
    }
}
