mod js_runtime;
mod js_runtime_resources;
mod js_runtime_scripts;
pub mod types;

pub use types::*;

pub type DrawioRenderOps = DrawioRendererOps;

use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramBlock, DiagramResult};
use js_runtime::DrawioJsRuntimeOps;
use std::path::PathBuf;

const DRAWIO_DOWNLOAD_URL: &str = "https://github.com/jgraph/drawio";

impl DrawioRendererOps {
    pub fn default_install_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".local").join("katana").join("drawio.min.js"))
    }

    pub fn resolve_drawio_js() -> PathBuf {
        #[allow(clippy::single_match)]
        match std::env::var("DRAWIO_JS") {
            Ok(path) => return PathBuf::from(path),
            Err(_) => {}
        }

        Self::default_install_path().unwrap_or_else(|| PathBuf::from("drawio.min.js"))
    }

    pub fn find_drawio_js() -> Option<PathBuf> {
        let path = Self::resolve_drawio_js();
        path.exists().then_some(path)
    }

    pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
        let Some(drawio_js) = Self::find_drawio_js() else {
            return DiagramResult::NotInstalled {
                kind: "Draw.io".to_string(),
                download_url: DRAWIO_DOWNLOAD_URL.to_string(),
                install_path: Self::resolve_drawio_js(),
            };
        };

        let preset = DiagramColorPreset::current();
        match DrawioJsRuntimeOps::render(&block.source, &drawio_js, preset) {
            Ok(svg) => DiagramResult::Ok(svg),
            Err(error) => {
                tracing::warn!("Draw.io JavaScript rendering failed: {error}");
                DiagramResult::Err {
                    source: block.source.clone(),
                    error: "not supported".to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
