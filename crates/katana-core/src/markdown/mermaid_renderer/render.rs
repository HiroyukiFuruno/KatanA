use super::types::MermaidRenderOps;
use super::web::MermaidWebRendererOps;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramBlock, DiagramResult};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

const MERMAID_DOWNLOAD_URL: &str = "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js";

impl MermaidRenderOps {
    pub fn render_mermaid(block: &DiagramBlock) -> DiagramResult {
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
        let cache_file = Self::cache_file_path(&block.source, &preset);
        let _ = std::fs::create_dir_all(cache_file.parent().unwrap_or_else(|| Path::new(".")));

        if let Some(data) = std::fs::read(&cache_file).ok() {
            return DiagramResult::OkPng(data);
        }

        match MermaidWebRendererOps::render_with_headless_chrome(
            &block.source,
            &mermaid_js,
            &preset,
        ) {
            Ok(data) => {
                let _ = std::fs::write(&cache_file, &data);
                DiagramResult::OkPng(data)
            }
            Err(e) => DiagramResult::Err {
                source: block.source.clone(),
                error: format!("Mermaid rendering failed: {e}"),
            },
        }
    }

    fn cache_file_path(source: &str, preset: &DiagramColorPreset) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        "mermaid-png-theme-v5".hash(&mut hasher);
        source.hash(&mut hasher);
        preset.mermaid_theme.hash(&mut hasher);
        preset.background.hash(&mut hasher);
        preset.text.hash(&mut hasher);
        preset.fill.hash(&mut hasher);
        preset.stroke.hash(&mut hasher);
        preset.arrow.hash(&mut hasher);
        std::env::temp_dir()
            .join("katana_mermaid_cache")
            .join(format!("{:016x}.png", hasher.finish()))
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
        let first = MermaidRenderOps::cache_file_path("graph TD;A", &preset_default);
        let mut dark = preset_default.clone();
        dark.mermaid_theme = "dark";
        let second = MermaidRenderOps::cache_file_path("graph TD;A", &dark);
        assert_ne!(first, second);
    }
}
