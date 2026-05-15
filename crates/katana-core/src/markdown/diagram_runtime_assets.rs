#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramRuntimeAssetKind {
    Mermaid,
    DrawIo,
}

pub struct DiagramRuntimeAssetOps;

impl DiagramRuntimeAssetOps {
    pub fn resolve_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        katana_diagram_renderer::RuntimePathResolver::resolve(Self::kdr_kind(kind), None).ok()
    }

    pub fn find_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        let path = Self::resolve_path(kind)?;
        path.exists().then_some(path)
    }

    fn kdr_kind(kind: DiagramRuntimeAssetKind) -> katana_diagram_renderer::DiagramKind {
        match kind {
            DiagramRuntimeAssetKind::Mermaid => katana_diagram_renderer::DiagramKind::Mermaid,
            DiagramRuntimeAssetKind::DrawIo => katana_diagram_renderer::DiagramKind::Drawio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kdr_kind_maps_all_runtime_asset_kinds() {
        assert!(matches!(
            DiagramRuntimeAssetOps::kdr_kind(DiagramRuntimeAssetKind::Mermaid),
            katana_diagram_renderer::DiagramKind::Mermaid
        ));
        assert!(matches!(
            DiagramRuntimeAssetOps::kdr_kind(DiagramRuntimeAssetKind::DrawIo),
            katana_diagram_renderer::DiagramKind::Drawio
        ));
    }
}
