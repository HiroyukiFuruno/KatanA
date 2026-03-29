use comrak::{markdown_to_html, ComrakOptions};

use super::diagram::{self, DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult};
use super::drawio_renderer;
use super::mermaid_renderer;
use super::plantuml_renderer;

use super::fence::transform_diagram_blocks;

/// Production renderer: delegates each diagram block type to the actual subprocess / XML parser.
#[derive(Debug, Default)]
pub struct KatanaRenderer;

impl DiagramRenderer for KatanaRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        match block.kind {
            DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
            DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
            DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
        }
    }
}

/// The result of rendering a Markdown buffer.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

/// Errors that may arise during Markdown rendering.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
    #[error("Export failed: {0}")]
    ExportFailed(String),
}

/// Build default `comrak` options with GFM extensions enabled.
pub fn gfm_options() -> ComrakOptions<'static> {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    // WHY: Required to output custom HTML (markup after diagram block conversion) as-is.
    opts.render.unsafe_ = true;
    opts
}

/// Renders Markdown to HTML using the production `KatanaRenderer`.
pub fn render_with_katana_renderer(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &KatanaRenderer)
}

/// Render Markdown to HTML, routing diagram fences through `renderer`.
pub fn render<R: DiagramRenderer>(
    source: &str,
    renderer: &R,
) -> Result<RenderOutput, MarkdownError> {
    let transformed = transform_diagram_blocks(source, renderer);
    let html = markdown_to_html(&transformed, &gfm_options());
    Ok(RenderOutput { html })
}

/// Convenience render using the no-op diagram renderer.
pub fn render_basic(source: &str) -> Result<RenderOutput, MarkdownError> {
    render(source, &diagram::NoOpRenderer)
}
