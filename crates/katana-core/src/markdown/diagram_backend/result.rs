use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::markdown::DiagramResult;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagramBackendOutput {
    HtmlFragment(String),
    Png(Vec<u8>),
}

impl DiagramBackendOutput {
    pub fn into_diagram_result(self) -> DiagramResult {
        match self {
            Self::HtmlFragment(html) => DiagramResult::Ok(html),
            Self::Png(bytes) => DiagramResult::OkPng(bytes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramBackendError {
    RenderFailed {
        message: String,
    },
    CommandNotFound {
        tool_name: String,
        install_hint: String,
    },
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: PathBuf,
    },
}

impl DiagramBackendError {
    pub fn into_diagram_result(self, source: impl Into<String>) -> DiagramResult {
        match self {
            Self::RenderFailed { message } => DiagramResult::Err {
                source: source.into(),
                error: message,
            },
            Self::CommandNotFound {
                tool_name,
                install_hint,
            } => DiagramResult::CommandNotFound {
                tool_name,
                install_hint,
                source: source.into(),
            },
            Self::NotInstalled {
                kind,
                download_url,
                install_path,
            } => DiagramResult::NotInstalled {
                kind,
                download_url,
                install_path,
            },
        }
    }
}

pub type DiagramBackendRenderResult = Result<DiagramBackendOutput, DiagramBackendError>;
