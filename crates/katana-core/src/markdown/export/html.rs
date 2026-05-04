use super::html_template::HtmlExportTemplate;
use super::types::{
    ExportError, ExportFormat, ExportInput, ExportOutput, ExporterTrait, HtmlExporter,
};
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramRenderer, MarkdownError, MarkdownRenderOps};

impl HtmlExporter {
    pub fn export_markdown_to_html(
        &self,
        source: &str,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        let renderer = crate::markdown::KatanaRenderer;
        Self::export(source, &renderer, preset, base_dir)
    }

    pub fn export<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        let output = MarkdownRenderOps::render(source, renderer)?;
        let css = HtmlExportTemplate::generate_css(preset);
        let body = match base_dir {
            Some(dir) => HtmlExportTemplate::resolve_relative_paths(&output.html, dir),
            None => output.html,
        };

        Ok(HtmlExportTemplate::assemble_html_document(&css, &body))
    }
}

static HTML_FORMATS: &[ExportFormat] = &[ExportFormat::Html];

impl ExporterTrait for HtmlExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        if input.format != ExportFormat::Html {
            return Err(ExportError::UnsupportedFormat);
        }
        std::fs::write(&input.output_path, input.html_source.as_bytes())
            .map_err(|e| ExportError::IoError(e.to_string()))?;
        Ok(ExportOutput {
            output_path: input.output_path.clone(),
            format: ExportFormat::Html,
        })
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        HTML_FORMATS
    }
}
