pub use katana_canvas_forge::exporter::{
    ExportConfig, ExportError, ExportFormat, ExportInput, ExportOutput, ExporterTrait, PaperSize,
};

use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramRenderer, MarkdownError, MarkdownRenderOps};

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
        let kcf_preset = kcf_preset(preset);
        let css = katana_canvas_forge::exporter::HtmlExporter::css_for_preset(&kcf_preset);
        let body = match base_dir {
            Some(dir) => katana_canvas_forge::exporter::HtmlExporter::resolve_relative_paths(
                &output.html,
                dir,
            ),
            None => output.html,
        };

        Ok(katana_canvas_forge::exporter::HtmlExporter::assemble_document(&css, &body))
    }
}

impl ExporterTrait for HtmlExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        katana_canvas_forge::exporter::ExporterTrait::export(
            &katana_canvas_forge::exporter::HtmlExporter,
            input,
        )
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        katana_canvas_forge::exporter::ExporterTrait::supported_formats(
            &katana_canvas_forge::exporter::HtmlExporter,
        )
    }
}

impl ImageExporter {
    pub fn is_available() -> bool {
        katana_canvas_forge::exporter::ImageExporter::is_available()
    }
}

impl ExporterTrait for ImageExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        katana_canvas_forge::exporter::ExporterTrait::export(
            &katana_canvas_forge::exporter::ImageExporter,
            input,
        )
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        katana_canvas_forge::exporter::ExporterTrait::supported_formats(
            &katana_canvas_forge::exporter::ImageExporter,
        )
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }
}

impl PdfExporter {
    pub fn is_available() -> bool {
        katana_canvas_forge::exporter::PdfExporter::is_available()
    }
}

impl ExporterTrait for PdfExporter {
    fn export(&self, input: &ExportInput) -> Result<ExportOutput, ExportError> {
        katana_canvas_forge::exporter::ExporterTrait::export(
            &katana_canvas_forge::exporter::PdfExporter,
            input,
        )
    }

    fn supported_formats(&self) -> &[ExportFormat] {
        katana_canvas_forge::exporter::ExporterTrait::supported_formats(
            &katana_canvas_forge::exporter::PdfExporter,
        )
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }
}

fn kcf_preset(
    preset: &DiagramColorPreset,
) -> katana_canvas_forge::markdown::color_preset::DiagramColorPreset {
    katana_canvas_forge::markdown::color_preset::DiagramColorPreset {
        background: preset.background,
        text: preset.text,
        fill: preset.fill,
        stroke: preset.stroke,
        arrow: preset.arrow,
        drawio_label_color: preset.drawio_label_color,
        mermaid_theme: preset.mermaid_theme,
        plantuml_class_bg: preset.plantuml_class_bg,
        plantuml_note_bg: preset.plantuml_note_bg,
        plantuml_note_text: preset.plantuml_note_text,
        syntax_theme_dark: preset.syntax_theme_dark,
        syntax_theme_light: preset.syntax_theme_light,
        preview_text: preset.preview_text,
        proportional_font_candidates: preset.proportional_font_candidates.clone(),
        monospace_font_candidates: preset.monospace_font_candidates.clone(),
        emoji_font_candidates: preset.emoji_font_candidates.clone(),
        editor_font_size: preset.editor_font_size,
    }
}
