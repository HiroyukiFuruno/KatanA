#[path = "renderer_cache.rs"]
mod renderer_cache;
#[path = "renderer_dispatch.rs"]
mod renderer_dispatch;
#[path = "renderer_png.rs"]
mod renderer_png;

use super::types::*;
use katana_core::markdown::{
    DiagramBlock, DiagramKind, DiagramResult,
    svg_rasterize::{RasterizedSvg, SvgRasterizeOps},
};

pub use super::types::RendererLogicOps;

impl RendererLogicOps {
    #[cfg(test)]
    pub fn render_diagram(
        kind: &DiagramKind,
        source: &str,
        source_lines: usize,
    ) -> RenderedSection {
        let block = DiagramBlock {
            kind: kind.clone(),
            source: source.to_string(),
        };
        let result = Self::dispatch_renderer(&block);
        Self::map_diagram_result(kind, source, result, source_lines)
    }

    pub fn map_diagram_result(
        kind: &DiagramKind,
        source: &str,
        result: DiagramResult,
        source_lines: usize,
    ) -> RenderedSection {
        match result {
            DiagramResult::Ok(html) => Self::try_rasterize(kind, source, &html, source_lines),
            DiagramResult::OkPng(bytes) => {
                Self::decode_png_to_section(kind, source, bytes, source_lines)
            }
            DiagramResult::Err { source, error } => {
                Self::render_error_section(kind, source, error, source_lines)
            }
            DiagramResult::CommandNotFound {
                tool_name,
                install_hint,
                source,
            } => RenderedSection::CommandNotFound {
                tool_name,
                install_hint,
                _source: source,
                source_lines,
            },
            DiagramResult::NotInstalled {
                kind: k,
                download_url,
                install_path,
            } => RenderedSection::NotInstalled {
                kind: k,
                download_url,
                install_path,
                source_lines,
            },
        }
    }

    pub(crate) fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
        renderer_dispatch::dispatch_renderer(block)
    }

    fn render_error_section(
        kind: &DiagramKind,
        source: String,
        error: String,
        source_lines: usize,
    ) -> RenderedSection {
        if matches!(kind, DiagramKind::Mermaid | DiagramKind::DrawIo) {
            return RenderedSection::Markdown(
                Self::diagram_error_markdown(kind, &source),
                source_lines,
            );
        }
        RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
            source_lines,
        }
    }

    fn diagram_error_markdown(kind: &DiagramKind, source: &str) -> String {
        format!(
            "not supported\n\n```{}\n{}\n```",
            Self::diagram_code_language(kind),
            source
        )
    }

    fn diagram_code_language(kind: &DiagramKind) -> &'static str {
        match kind {
            DiagramKind::Mermaid => "mermaid",
            DiagramKind::PlantUml => "plantuml",
            DiagramKind::DrawIo => "drawio",
        }
    }

    pub fn try_rasterize(
        kind: &DiagramKind,
        source: &str,
        html: &str,
        source_lines: usize,
    ) -> RenderedSection {
        let Some(svg) = Self::extract_svg(html) else {
            return RenderedSection::Error {
                kind: format!("{kind:?}"),
                _source: source.to_string(),
                message: "Failed to extract SVG".to_string(),
                source_lines,
            };
        };
        match SvgRasterizeOps::rasterize_svg(svg, DIAGRAM_SVG_DISPLAY_SCALE) {
            Ok(img) => RenderedSection::Image {
                svg_data: img,
                alt: diagram_alt_text(kind, source),
                source_lines,
            },
            Err(e) => RenderedSection::Error {
                kind: format!("{kind:?}"),
                _source: source.to_string(),
                message: e.to_string(),
                source_lines,
            },
        }
    }

    pub fn extract_svg(html: &str) -> Option<&str> {
        let start = html.find("<svg")?;
        let end = html.rfind("</svg>")? + "</svg>".len();
        Some(&html[start..end])
    }

    pub fn decode_png_to_section(
        kind: &DiagramKind,
        source: &str,
        bytes: Vec<u8>,
        source_lines: usize,
    ) -> RenderedSection {
        renderer_png::RendererPngDecoder::decode_png_to_section(kind, source, bytes, source_lines)
    }

    pub fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
        renderer_png::RendererPngDecoder::decode_png_rgba(bytes)
    }
}

fn diagram_alt_text(kind: &DiagramKind, source: &str) -> String {
    if kind.is_zenuml_source(source) {
        return "ZenUML diagram".to_string();
    }
    format!("{kind:?} diagram")
}
