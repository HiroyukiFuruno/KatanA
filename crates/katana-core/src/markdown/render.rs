use comrak::{ComrakOptions, markdown_to_html};

use super::drawio_renderer;
use super::fence::MarkdownFenceOps;
use super::mermaid_renderer;
use super::plantuml_renderer;
use super::types::*;

impl DiagramRenderer for KatanaRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        match block.kind {
            DiagramKind::Mermaid => mermaid_renderer::MermaidRenderOps::render_mermaid(block),
            DiagramKind::PlantUml => plantuml_renderer::PlantUmlRendererOps::render_plantuml(block),
            DiagramKind::DrawIo => drawio_renderer::DrawioRendererOps::render_drawio(block),
        }
    }
}

impl MarkdownRenderOps {
    pub fn gfm_options() -> ComrakOptions<'static> {
        let mut opts = ComrakOptions::default();
        opts.extension.strikethrough = true;
        opts.extension.table = true;
        opts.extension.autolink = true;
        opts.extension.tasklist = true;
        opts.extension.footnotes = true;
        opts.render.unsafe_ = true;
        opts
    }

    pub fn render_basic(source: &str) -> Result<RenderOutput, MarkdownError> {
        Self::render(source, &NoOpRenderer)
    }

    pub fn render_with_katana_renderer(source: &str) -> Result<RenderOutput, MarkdownError> {
        Self::render(source, &KatanaRenderer)
    }

    pub fn render<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
    ) -> Result<RenderOutput, MarkdownError> {
        let transformed = MarkdownFenceOps::transform_diagram_blocks(source, renderer);
        let html = markdown_to_html(&transformed, &Self::gfm_options());
        Ok(RenderOutput { html })
    }

    pub fn transform_only<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
    ) -> Result<RenderOutput, MarkdownError> {
        let html = MarkdownFenceOps::transform_diagram_blocks(source, renderer);
        Ok(RenderOutput { html })
    }
}
