use crate::markdown::DiagramKind;

#[derive(Debug, Clone)]
pub enum PreviewSection {
    Markdown(String),
    Diagram {
        kind: DiagramKind,
        source: String,
        lines: usize,
    },
    LocalImage {
        path: String,
        alt: String,
        lines: usize,
    },
}

pub struct PreviewFlattenOps;
pub struct PreviewSectionOps;
pub struct ImagePreviewOps;
pub struct HtmlPreviewOps;
pub struct ImageSectionOps;
pub struct DiagramSectionOps;
pub struct MathPreviewOps;
