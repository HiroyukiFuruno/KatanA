use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewSourceRange {
    pub start_byte: usize,
    pub end_byte: usize,
}

impl PreviewSourceRange {
    pub fn new(start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewRenderedId {
    pub value: String,
}

impl PreviewRenderedId {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewHeadingAnchor {
    pub id: String,
    pub level: u8,
    pub title: String,
    pub source_range: PreviewSourceRange,
    pub rendered_id: PreviewRenderedId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewBlockAnchor {
    pub id: String,
    pub kind: PreviewBlockKind,
    pub source_range: PreviewSourceRange,
    pub rendered_id: PreviewRenderedId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreviewBlockKind {
    Paragraph,
    Heading,
    CodeBlock,
    Diagram,
    Table,
    Math,
    Html,
    Image,
    List,
    Quote,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewActionHook {
    pub id: String,
    pub source_range: Option<PreviewSourceRange>,
    pub action: PreviewAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreviewAction {
    OpenLink { href: String },
    CopyCode { code: String },
    ToggleTask { checked: bool },
    SelectSource { range: PreviewSourceRange },
    RefreshDiagram { block_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PreviewRenderMetadata {
    pub headings: Vec<PreviewHeadingAnchor>,
    pub blocks: Vec<PreviewBlockAnchor>,
    pub actions: Vec<PreviewActionHook>,
    pub attributes: BTreeMap<String, String>,
}
