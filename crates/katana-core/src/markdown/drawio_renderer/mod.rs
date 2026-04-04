pub mod edge;
pub mod svg;
pub mod types;
pub mod utils;
pub mod vertex;

pub use types::*;

/// Compatibility alias.
pub type DrawioRenderOps = DrawioRendererOps;

use crate::markdown::{DiagramBlock, DiagramResult};

impl DrawioRendererOps {
    pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
        match DrawioSvgOps::render_drawio_to_svg(&block.source) {
            Ok(svg) => {
                DiagramResult::Ok(format!(r#"<div class="katana-diagram drawio">{svg}</div>"#))
            }
            Err(e) => DiagramResult::Err {
                source: block.source.clone(),
                error: e.to_string(),
            },
        }
    }
}
