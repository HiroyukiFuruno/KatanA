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
