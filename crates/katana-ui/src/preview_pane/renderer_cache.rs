use super::super::types::RendererLogicOps;
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::{
    DiagramBackendCacheKey, DiagramBackendFactory, DiagramBackendInput, DiagramBackendLanguage,
    DiagramDocumentContext, DiagramKind, DiagramRenderOptions, DiagramThemeSnapshot,
};
use katana_platform::cache::PersistentKey;
use std::hash::{Hash, Hasher};

impl RendererLogicOps {
    pub fn get_cache_key(
        md_file_path: &std::path::Path,
        kind: &DiagramKind,
        source: &str,
    ) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let backend_language = Self::backend_language(kind);
        let backend = DiagramBackendFactory::create(backend_language.clone());
        let input = Self::backend_input(md_file_path, backend_language, source);
        DiagramBackendCacheKey::new(backend.id().clone(), backend.version().clone(), &input)
            .hash(&mut hasher);
        Self::persistent_key(md_file_path, kind, format!("{:x}", hasher.finish()))
    }

    fn backend_language(kind: &DiagramKind) -> DiagramBackendLanguage {
        match kind {
            DiagramKind::Mermaid => DiagramBackendLanguage::Mermaid,
            DiagramKind::PlantUml => DiagramBackendLanguage::PlantUml,
            DiagramKind::DrawIo => DiagramBackendLanguage::DrawIo,
        }
    }

    fn backend_input(
        md_file_path: &std::path::Path,
        language: DiagramBackendLanguage,
        source: &str,
    ) -> DiagramBackendInput {
        let is_dark = DiagramColorPreset::is_dark_mode();
        DiagramBackendInput {
            language,
            source: source.to_string(),
            options: DiagramRenderOptions::default(),
            theme: DiagramThemeSnapshot::from_preset(
                if is_dark { "dark" } else { "light" },
                is_dark,
                DiagramColorPreset::current(),
            ),
            document: DiagramDocumentContext::Detached {
                display_name: md_file_path.to_string_lossy().to_string(),
            },
        }
    }

    fn persistent_key(
        md_file_path: &std::path::Path,
        kind: &DiagramKind,
        source_hash: String,
    ) -> String {
        PersistentKey::Diagram {
            document_path: md_file_path.to_path_buf(),
            diagram_kind: kind.display_name().to_string(),
            theme: if DiagramColorPreset::is_dark_mode() {
                "dark".to_string()
            } else {
                "light".to_string()
            },
            source_hash,
        }
        .to_raw_key()
        .unwrap_or_default()
    }
}
