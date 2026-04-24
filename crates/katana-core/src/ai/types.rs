use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct AiRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub params: Vec<Param>,
}

impl AiRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            params: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiResponse {
    pub content: String,
    pub metadata: Vec<Param>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiCapabilities {
    pub chat: bool,
    pub autofix: bool,
}

impl AiCapabilities {
    pub const fn chat_and_autofix() -> Self {
        Self {
            chat: true,
            autofix: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModel {
    pub name: String,
    pub size_bytes: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("No AI provider is configured")]
    NotConfigured,

    #[error("No AI model is selected")]
    ModelNotSelected,

    #[error("Provider request failed: {0}")]
    RequestFailed(String),

    #[error("Provider returned an invalid response: {0}")]
    InvalidResponse(String),
}
