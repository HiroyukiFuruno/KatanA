#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramRuntimeAssetKind {
    Mermaid,
    DrawIo,
}

pub struct DiagramRuntimeAssetOps;

impl DiagramRuntimeAssetOps {
    pub fn resolve_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        katana_canvas_forge::RuntimePathResolver::resolve(Self::kcf_kind(kind), None).ok()
    }

    pub fn find_path(kind: DiagramRuntimeAssetKind) -> Option<std::path::PathBuf> {
        let path = Self::resolve_path(kind)?;
        path.exists().then_some(path)
    }

    fn kcf_kind(kind: DiagramRuntimeAssetKind) -> katana_canvas_forge::DiagramKind {
        match kind {
            DiagramRuntimeAssetKind::Mermaid => katana_canvas_forge::DiagramKind::Mermaid,
            DiagramRuntimeAssetKind::DrawIo => katana_canvas_forge::DiagramKind::Drawio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kcf_kind_maps_all_runtime_asset_kinds() {
        assert!(matches!(
            DiagramRuntimeAssetOps::kcf_kind(DiagramRuntimeAssetKind::Mermaid),
            katana_canvas_forge::DiagramKind::Mermaid
        ));
        assert!(matches!(
            DiagramRuntimeAssetOps::kcf_kind(DiagramRuntimeAssetKind::DrawIo),
            katana_canvas_forge::DiagramKind::Drawio
        ));
    }
}
