use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramKind {
    Mermaid,
    PlantUml,
    DrawIo,
}

#[derive(Debug, Clone)]
pub struct DiagramBlock {
    pub kind: DiagramKind,
    pub source: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DiagramValidationError {
    #[error("{kind} block has empty source")]
    EmptySource { kind: &'static str },

    #[error("{kind} block is missing required delimiters: {message}")]
    MissingDelimiters { kind: &'static str, message: String },

    #[error("{kind} block uses an unsupported encoding: {message}")]
    UnsupportedEncoding { kind: &'static str, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagramResult {
    Ok(String),
    OkPng(Vec<u8>),
    Err {
        source: String,
        error: String,
    },
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        source: String,
    },
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
    },
}

pub struct NoOpRenderer;

pub struct RenderOptions {
    pub allow_raw_html: bool,
    pub convert_diagrams: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
    #[error("Export failed: {0}")]
    ExportFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

pub struct RasterizeOps;
pub struct MarkdownRenderOps;

pub trait DiagramRenderer: Send + Sync {
    fn render(&self, block: &DiagramBlock) -> DiagramResult;
}

#[derive(Debug, Default)]
pub struct KatanaRenderer;
