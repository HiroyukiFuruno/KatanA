use super::js_runtime_resources::{DrawioResource, DrawioResourceCatalog};
use super::js_runtime_scripts::DrawioRuntimeScripts;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_js_runtime::DiagramV8Runtime;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

static BUNDLE_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<str>>>> = OnceLock::new();

pub(super) struct DrawioJsRuntimeOps;

impl DrawioJsRuntimeOps {
    pub(super) fn render(
        source: &str,
        drawio_js: &Path,
        preset: &DiagramColorPreset,
    ) -> Result<String, String> {
        let bundle = read_drawio_bundle(drawio_js)?;
        let request = DrawioRenderRequest {
            source,
            dark_mode: DiagramColorPreset::is_dark_mode(),
            background: preset.background,
            resources: DrawioResourceCatalog::builtin(source),
        };
        let request_json = serde_json::to_string(&request).map_err(|err| err.to_string())?;
        let scripts = DrawioRuntimeScripts::build(&bundle, &request_json);
        let svg = DiagramV8Runtime::render(&scripts)?;
        ensure_svg(&svg)?;
        Ok(svg)
    }
}

#[derive(Serialize)]
struct DrawioRenderRequest<'a> {
    source: &'a str,
    dark_mode: bool,
    background: &'a str,
    resources: Vec<DrawioResource>,
}

fn read_drawio_bundle(drawio_js: &Path) -> Result<Arc<str>, String> {
    let cache = BUNDLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let path = drawio_js.to_path_buf();
    if let Some(bundle) = cache.lock().map_err(|err| err.to_string())?.get(&path) {
        return Ok(bundle.clone());
    }

    let bundle = std::fs::read_to_string(drawio_js)
        .map_err(|err| format!("Failed to read Draw.io JavaScript bundle: {err}"))?;
    let bundle = Arc::<str>::from(bundle);
    cache
        .lock()
        .map_err(|err| err.to_string())?
        .insert(path, bundle.clone());
    Ok(bundle)
}

fn ensure_svg(svg: &str) -> Result<(), String> {
    if svg.contains("<svg") && svg.contains("</svg>") {
        return Ok(());
    }
    Err("Draw.io JavaScript did not return SVG markup".to_string())
}
