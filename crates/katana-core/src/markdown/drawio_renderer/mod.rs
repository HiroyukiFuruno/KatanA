//! Draw.io (mxGraph) XML to SVG conversion renderer.
//!
//! MVP supported scope:
//! - Only accepts uncompressed `<mxfile>` / `<mxGraphModel>` XML.
//! - Converts `vertex` (rectangle/rounded rectangle) and `edge` (straight arrow) of `<mxCell>` to SVG.
//! - Minimal style parsing (`rounded`, `ellipse`, `label`, `fillColor`, `strokeColor`).
//! - Unsupported elements are skipped, rendering only supported ones.

use xmltree::Element;

use super::diagram::{DiagramBlock, DiagramResult};

pub mod edge;
pub mod parse;
pub mod svg;
pub mod utils;
pub mod vertex;

/// Converts Draw.io XML to an SVG HTML fragment.
pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
    match convert_xml_to_svg(&block.source) {
        Ok(svg) => DiagramResult::Ok(format!(r#"<div class="katana-diagram drawio">{svg}</div>"#)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// Parses XML and returns an SVG string.
fn convert_xml_to_svg(xml: &str) -> Result<String, String> {
    let root = Element::parse(xml.as_bytes()).map_err(|e| format!("XML parse error: {e}"))?;
    let model = parse::extract_graph_model(&root)?;
    let cells = parse::collect_cells(model);
    let (w, h) = parse::estimate_canvas_size(&cells);
    Ok(svg::build_svg(&cells, w, h))
}
