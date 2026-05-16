use crate::preview_pane::types::RenderedSection;
use katana_core::markdown::DiagramKind;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) struct RendererPngDecoder;

impl RendererPngDecoder {
    pub(crate) fn decode_png_to_section(
        kind: &DiagramKind,
        source: &str,
        bytes: Vec<u8>,
        source_lines: usize,
    ) -> RenderedSection {
        match Self::decode_png_rgba(&bytes) {
            Ok(rasterized) => RenderedSection::Image {
                svg_data: rasterized,
                alt: diagram_alt_text(kind, source),
                source_lines,
            },
            Err(e) => RenderedSection::Error {
                kind: format!("{kind:?}"),
                _source: source.to_string(),
                message: format!("PNG decode failed: {e}"),
                source_lines,
            },
        }
    }

    pub(crate) fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
        let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
        let rgba = img.into_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(RasterizedSvg::new(
            width,
            height,
            width as f32,
            height as f32,
            rgba.into_raw(),
        ))
    }
}

fn diagram_alt_text(kind: &DiagramKind, source: &str) -> String {
    if kind.is_zenuml_source(source) {
        return "ZenUML diagram".to_string();
    }
    format!("{kind:?} diagram")
}
