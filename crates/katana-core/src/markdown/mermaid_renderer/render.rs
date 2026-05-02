use super::js_runtime::MermaidJsRuntimeOps;
use super::types::MermaidRenderOps;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_runtime::DiagramRuntimeMode;
use crate::markdown::{DiagramBlock, DiagramResult};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MERMAID_DOWNLOAD_URL: &str = "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js";
static MERMAID_SVG_RENDER_SEQUENCE: AtomicU64 = AtomicU64::new(1);

impl MermaidRenderOps {
    pub fn render_mermaid(block: &DiagramBlock) -> DiagramResult {
        if block.source.trim().is_empty() {
            return DiagramResult::Ok(String::new());
        }

        let Some(mermaid_js) =
            crate::markdown::mermaid_renderer::MermaidBinaryOps::find_mermaid_js()
        else {
            return DiagramResult::NotInstalled {
                kind: "Mermaid".to_string(),
                download_url: MERMAID_DOWNLOAD_URL.to_string(),
                install_path:
                    crate::markdown::mermaid_renderer::MermaidBinaryOps::resolve_mermaid_js(),
            };
        };

        let preset = DiagramColorPreset::current();
        let mode = DiagramRuntimeMode::current();
        let cache_file = Self::cache_file_path(&block.source, preset, mode);
        let _ = std::fs::create_dir_all(cache_file.parent().unwrap_or_else(|| Path::new(".")));

        Self::render_svg(block, &mermaid_js, preset, &cache_file)
    }

    pub fn cache_profile() -> &'static str {
        DiagramRuntimeMode::current().mermaid_cache_profile()
    }

    fn render_svg(
        block: &DiagramBlock,
        mermaid_js: &Path,
        preset: &DiagramColorPreset,
        cache_file: &Path,
    ) -> DiagramResult {
        if let Some(svg) = Self::read_cached_svg(cache_file) {
            return DiagramResult::Ok(Self::unique_svg_instance(svg));
        }
        match MermaidJsRuntimeOps::render(&block.source, mermaid_js, preset) {
            Ok(svg) => {
                let _ = std::fs::write(cache_file, &svg);
                DiagramResult::Ok(Self::unique_svg_instance(svg))
            }
            Err(e) => {
                tracing::warn!("Mermaid JavaScript rendering failed: {e}");
                DiagramResult::Err {
                    source: block.source.clone(),
                    error: "not supported".to_string(),
                }
            }
        }
    }

    fn cache_file_path(
        source: &str,
        preset: &DiagramColorPreset,
        mode: DiagramRuntimeMode,
    ) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        "mermaid-render-theme-v119-sankey-i18n-node-identity".hash(&mut hasher);
        mode.mermaid_cache_profile().hash(&mut hasher);
        source.hash(&mut hasher);
        preset.mermaid_theme.hash(&mut hasher);
        preset.background.hash(&mut hasher);
        preset.text.hash(&mut hasher);
        preset.fill.hash(&mut hasher);
        preset.stroke.hash(&mut hasher);
        preset.arrow.hash(&mut hasher);
        std::env::temp_dir()
            .join("katana_mermaid_cache")
            .join(format!(
                "{:016x}.{}",
                hasher.finish(),
                mode.mermaid_cache_extension()
            ))
    }

    fn read_cached_svg(cache_file: &Path) -> Option<String> {
        std::fs::read_to_string(cache_file).ok()
    }

    fn unique_svg_instance(svg: String) -> String {
        let Some(root_id) = Self::root_svg_id(&svg) else {
            return svg;
        };
        let sequence = MERMAID_SVG_RENDER_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let unique_id = format!("{root_id}-{sequence:016x}");
        svg.replace(&root_id, &unique_id)
    }

    fn root_svg_id(svg: &str) -> Option<String> {
        let svg_start = svg.find("<svg")?;
        let open_end = svg_start + svg[svg_start..].find('>')?;
        let marker = r#"id="katana-mermaid-svg-"#;
        let start = svg_start + svg[svg_start..open_end].find(marker)? + r#"id=""#.len();
        let end = start + svg[start..open_end].find('"')?;
        Some(svg[start..end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn render_mermaid_returns_not_installed_when_missing_js() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let missing_js = temp_dir.path().join("missing-mermaid.js");
        unsafe { std::env::set_var("MERMAID_JS", &missing_js) };

        let block = crate::markdown::DiagramBlock {
            kind: crate::markdown::DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = MermaidRenderOps::render_mermaid(&block);
        unsafe { std::env::remove_var("MERMAID_JS") };
        assert!(matches!(result, DiagramResult::NotInstalled { .. }));
    }

    #[test]
    fn cache_file_path_changes_with_theme() {
        let preset_default = DiagramColorPreset::default();
        let mode = DiagramRuntimeMode::current();
        let first = MermaidRenderOps::cache_file_path("graph TD;A", &preset_default, mode);
        let mut dark = preset_default.clone();
        dark.mermaid_theme = "dark";
        let second = MermaidRenderOps::cache_file_path("graph TD;A", &dark, mode);
        assert_ne!(first, second);
    }

    #[test]
    fn cache_file_path_uses_rust_managed_svg_profile() {
        let preset = DiagramColorPreset::default();
        let path =
            MermaidRenderOps::cache_file_path("graph TD;A", &preset, DiagramRuntimeMode::current());
        assert_eq!(path.extension().and_then(|it| it.to_str()), Some("svg"));
    }

    #[test]
    fn render_mermaid_skips_empty_source() {
        let block = crate::markdown::DiagramBlock {
            kind: crate::markdown::DiagramKind::Mermaid,
            source: "\n\t ".to_string(),
        };

        assert!(
            matches!(MermaidRenderOps::render_mermaid(&block), DiagramResult::Ok(svg) if svg.is_empty())
        );
    }

    #[test]
    fn unique_svg_instance_handles_root_id_after_other_attributes() {
        let base_id = "katana-mermaid-svg-abcdef0123456789";
        let svg = format!(
            r##"<svg height="20" id="{base_id}" width="20"><style>#{base_id} [id$="-arrowhead"] path{{fill:#AAAAAA;}}</style><path marker-end="url(#{base_id})"></path></svg>"##
        );

        let result = MermaidRenderOps::unique_svg_instance(svg);
        let result_id = MermaidRenderOps::root_svg_id(&result).unwrap();

        assert_ne!(result_id, base_id);
        assert!(result_id.starts_with(base_id));
        assert!(!result.contains(r#"id="katana-mermaid-svg-abcdef0123456789""#));
        assert!(!result.contains("url(#katana-mermaid-svg-abcdef0123456789)"));
    }
}
