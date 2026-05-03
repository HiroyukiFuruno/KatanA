use super::js_runtime_scripts::MermaidRuntimeScripts;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_js_runtime::DiagramV8Runtime;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

static BUNDLE_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<str>>>> = OnceLock::new();

pub(super) struct MermaidJsRuntimeOps;

impl MermaidJsRuntimeOps {
    pub(super) fn render(
        source: &str,
        mermaid_js: &Path,
        preset: &DiagramColorPreset,
    ) -> Result<String, String> {
        let bundle = read_mermaid_bundle(mermaid_js)?;
        let request = MermaidRenderRequest::new(source, preset);
        let request_json = serde_json::to_string(&request).map_err(|err| err.to_string())?;
        let scripts = MermaidRuntimeScripts::build(&bundle, &request_json);
        let svg = DiagramV8Runtime::render(&scripts)?;
        ensure_svg(&svg)?;
        Ok(svg)
    }
}

#[derive(Serialize)]
struct MermaidRenderRequest<'a> {
    source: &'a str,
    #[serde(rename = "svgId")]
    svg_id: String,
    theme: &'a str,
    background: &'a str,
    fill: &'a str,
    text: &'a str,
    stroke: &'a str,
    arrow: &'a str,
}

impl<'a> MermaidRenderRequest<'a> {
    fn new(source: &'a str, preset: &'a DiagramColorPreset) -> Self {
        Self {
            source,
            svg_id: Self::svg_id(source, preset),
            theme: preset.mermaid_theme,
            background: preset.background,
            fill: preset.fill,
            text: preset.text,
            stroke: preset.stroke,
            arrow: preset.arrow,
        }
    }

    fn svg_id(source: &str, preset: &DiagramColorPreset) -> String {
        let mut hasher = DefaultHasher::new();
        "mermaid-svg-id-v1".hash(&mut hasher);
        source.hash(&mut hasher);
        preset.mermaid_theme.hash(&mut hasher);
        preset.background.hash(&mut hasher);
        preset.text.hash(&mut hasher);
        preset.fill.hash(&mut hasher);
        preset.stroke.hash(&mut hasher);
        preset.arrow.hash(&mut hasher);
        format!("katana-mermaid-svg-{:016x}", hasher.finish())
    }
}

fn read_mermaid_bundle(mermaid_js: &Path) -> Result<Arc<str>, String> {
    let cache = BUNDLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let path = mermaid_js.to_path_buf();
    if let Some(bundle) = cache.lock().map_err(|err| err.to_string())?.get(&path) {
        return Ok(bundle.clone());
    }

    let bundle = std::fs::read_to_string(mermaid_js)
        .map_err(|err| format!("Failed to read Mermaid.js bundle: {err}"))?;
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
    Err("Mermaid.js did not return SVG markup".to_string())
}
