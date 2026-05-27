#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramRuntimeAssetKind {
    Mermaid,
    DrawIo,
}

pub struct DiagramRuntimeAssetOps;

impl DiagramRuntimeAssetOps {
    pub fn resolve_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        katana_render_runtime::RuntimePathResolver::resolve(Self::krr_kind(kind), None).ok()
    }

    pub fn find_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        let path = Self::resolve_path(kind)?;
        path.exists().then_some(path)
    }

    fn krr_kind(kind: DiagramRuntimeAssetKind) -> katana_render_runtime::DiagramKind {
        match kind {
            DiagramRuntimeAssetKind::Mermaid => katana_render_runtime::DiagramKind::Mermaid,
            DiagramRuntimeAssetKind::DrawIo => katana_render_runtime::DiagramKind::Drawio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn krr_kind_maps_all_runtime_asset_kinds() {
        assert!(matches!(
            DiagramRuntimeAssetOps::krr_kind(DiagramRuntimeAssetKind::Mermaid),
            katana_render_runtime::DiagramKind::Mermaid
        ));
        assert!(matches!(
            DiagramRuntimeAssetOps::krr_kind(DiagramRuntimeAssetKind::DrawIo),
            katana_render_runtime::DiagramKind::Drawio
        ));
    }
}
