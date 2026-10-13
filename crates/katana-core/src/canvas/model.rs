use super::{BackgroundStyle, CanvasColor, EdgeEnd, EdgeSide};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanvasDocument {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    pub diagnostics: Vec<CanvasDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasNode {
    pub id: String,
    pub geometry: CanvasGeometry,
    pub color: Option<CanvasColor>,
    pub content: NodeContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeContent {
    Text {
        text: String,
    },
    File {
        file: String,
        subpath: Option<String>,
    },
    Link {
        url: String,
    },
    Group {
        label: Option<String>,
        background: Option<String>,
        background_style: Option<BackgroundStyle>,
    },
    Unsupported {
        node_type: String,
    },
}

impl NodeContent {
    pub fn file_label(&self) -> Option<&str> {
        match self {
            Self::File { file, .. } => file.rsplit(['/', '\\']).next(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub from_side: Option<EdgeSide>,
    pub from_end: EdgeEnd,
    pub to_node: String,
    pub to_side: Option<EdgeSide>,
    pub to_end: EdgeEnd,
    pub color: Option<CanvasColor>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasLocation {
    Document,
    Node(usize),
    Edge(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasDiagnostic {
    UnsupportedNodeType {
        index: usize,
        node_type: String,
    },
    UnsupportedFields {
        location: CanvasLocation,
        fields: Vec<String>,
    },
}
