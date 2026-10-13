mod kdv_adapter;
mod kdv_markdown_normalizer;
#[cfg(test)]
mod tests;
mod types;

use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramRenderer, MarkdownError};

pub use types::{
    ExportConfig, ExportError, ExportFormat, ExportInput, ExportOutput, ExporterTrait, PaperSize,
};

pub struct HtmlExporter;
pub struct ImageExporter;
pub struct PdfExporter;

impl HtmlExporter {
    pub fn export_markdown_to_html(
        &self,
        source: &str,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        kdv_adapter::KdvExportAdapter::export_html_string(source, preset, base_dir)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    pub fn export<R: DiagramRenderer>(
        source: &str,
        _renderer: &R,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        Self.export_markdown_to_html(source, preset, base_dir)
    }
}

impl ExporterTrait for HtmlExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        kdv_adapter::KdvExportAdapter::export_to_file(input)
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        &[ExportFormat::Html]
    }
}

impl ImageExporter {
    pub fn is_available() -> bool {
        true
    }
}

impl ExporterTrait for ImageExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        kdv_adapter::KdvExportAdapter::export_to_file(input)
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        &[ExportFormat::Png, ExportFormat::Jpeg]
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }
}

impl PdfExporter {
    pub fn is_available() -> bool {
        true
    }
}

impl ExporterTrait for PdfExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        kdv_adapter::KdvExportAdapter::export_to_file(input)
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        &[ExportFormat::Pdf]
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }
}
