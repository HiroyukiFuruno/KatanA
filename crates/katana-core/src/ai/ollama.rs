use super::{AiCapabilities, AiError, AiModel, AiProvider, AiRequest, AiResponse, Param};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";
pub const DEFAULT_OLLAMA_TIMEOUT_SECS: u64 = 30;

pub struct OllamaProvider {
    endpoint: String,
    model: String,
    agent: ureq::Agent,
}

impl OllamaProvider {
    pub const ID: &'static str = "ollama";

    pub fn new(endpoint: impl Into<String>, model: impl Into<String>, timeout_secs: u64) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(timeout_secs.max(1))))
            .build();
        let agent = config.into();
        Self {
            endpoint: Self::normalize_endpoint(endpoint.into()),
            model: model.into().trim().to_string(),
            agent,
        }
    }

    fn normalize_endpoint(endpoint: String) -> String {
        let trimmed = endpoint.trim().trim_end_matches('/');
        if trimmed.is_empty() {
            DEFAULT_OLLAMA_ENDPOINT.to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/{}", self.endpoint, path.trim_start_matches('/'))
    }

    fn selected_model<'a>(&'a self, request: &'a AiRequest) -> Result<&'a str, AiError> {
        let model = request.model.as_deref().unwrap_or(&self.model).trim();
        if model.is_empty() {
            Err(AiError::ModelNotSelected)
        } else {
            Ok(model)
        }
    }
}

impl AiProvider for OllamaProvider {
    fn id(&self) -> &str {
        Self::ID
    }

    fn display_name(&self) -> &str {
        "Ollama"
    }

    fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let model = self.selected_model(request)?;
        let body = OllamaGenerateRequest {
            model,
            prompt: &request.prompt,
            stream: false,
        };
        let mut response = self
            .agent
            .post(&self.api_url("/api/generate"))
            .send_json(&body)
            .map_err(|err| AiError::RequestFailed(err.to_string()))?;
        let body = response
            .body_mut()
            .read_json::<OllamaGenerateResponse>()
            .map_err(|err| AiError::InvalidResponse(err.to_string()))?;
        Ok(AiResponse {
            content: body.response,
            metadata: vec![
                Param {
                    key: "provider".to_string(),
                    value: Self::ID.to_string(),
                },
                Param {
                    key: "model".to_string(),
                    value: model.to_string(),
                },
            ],
        })
    }

    fn is_available(&self) -> bool {
        !self.model.is_empty() && self.list_models().is_ok()
    }

    fn capabilities(&self) -> AiCapabilities {
        AiCapabilities::chat_and_autofix()
    }

    fn list_models(&self) -> Result<Vec<AiModel>, AiError> {
        let mut response = self
            .agent
            .get(&self.api_url("/api/tags"))
            .call()
            .map_err(|err| AiError::RequestFailed(err.to_string()))?;
        let body = response
            .body_mut()
            .read_json::<OllamaTagsResponse>()
            .map_err(|err| AiError::InvalidResponse(err.to_string()))?;
        let mut models = body
            .models
            .into_iter()
            .map(|model| AiModel {
                name: model.name,
                size_bytes: model.size,
            })
            .collect::<Vec<_>>();
        models.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(models)
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_endpoint_uses_default() {
        let provider = OllamaProvider::new("  ", "llama3.2", 1);
        assert_eq!(
            provider.api_url("/api/tags"),
            format!("{DEFAULT_OLLAMA_ENDPOINT}/api/tags")
        );
    }

    #[test]
    fn endpoint_trailing_slash_is_removed() {
        let provider = OllamaProvider::new("http://localhost:11434/", "llama3.2", 1);
        assert_eq!(
            provider.api_url("/api/tags"),
            "http://localhost:11434/api/tags"
        );
    }

    #[test]
    fn execute_fails_before_network_without_model() {
        let provider = OllamaProvider::new(DEFAULT_OLLAMA_ENDPOINT, "", 1);
        let request = AiRequest::new("hello");
        assert!(matches!(
            provider.execute(&request),
            Err(AiError::ModelNotSelected)
        ));
        assert!(!provider.is_available());
    }
}
