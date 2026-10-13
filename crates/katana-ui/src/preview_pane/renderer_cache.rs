use super::super::diagram_cache::DiagramRenderCacheCoordinator;
use super::super::types::RendererLogicOps;
use katana_core::markdown::DiagramKind;

impl RendererLogicOps {
    pub fn get_cache_key(
        md_file_path: &std::path::Path,
        kind: &DiagramKind,
        source: &str,
    ) -> String {
        DiagramRenderCacheCoordinator::cache_key(md_file_path, kind, source)
    }
}
