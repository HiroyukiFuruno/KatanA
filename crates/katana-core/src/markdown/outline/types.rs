#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub index: usize,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorKind {
    Heading,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentAnchor {
    pub kind: AnchorKind,
    pub line_start: usize,
    pub line_end: usize,
}
