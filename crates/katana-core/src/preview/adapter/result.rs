use serde::{Deserialize, Serialize};

use super::{PreviewRenderMetadata, PreviewSourceRange};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewOutput {
    pub html: String,
    pub metadata: PreviewRenderMetadata,
    pub diagnostics: Vec<PreviewDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreviewDiagnostic {
    pub severity: PreviewDiagnosticSeverity,
    pub message: String,
    pub source_range: Option<PreviewSourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreviewDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewAdapterError {
    RenderFailed { message: String },
}

pub type PreviewAdapterResult = Result<PreviewOutput, PreviewAdapterError>;
