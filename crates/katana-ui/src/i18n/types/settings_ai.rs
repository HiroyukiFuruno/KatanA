use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsAiMessages {
    pub provider_section: String,
    pub endpoint: String,
    pub endpoint_hint: String,
    pub model: String,
    pub model_hint: String,
    pub model_required: String,
    pub timeout_secs: String,
    pub timeout_hint: String,
    pub lightweight_hint: String,
    pub capabilities_section: String,
    pub chat_enabled: String,
    pub autofix_enabled: String,
}

impl Default for SettingsAiMessages {
    fn default() -> Self {
        Self {
            provider_section: "Ollama".to_string(),
            endpoint: "Endpoint".to_string(),
            endpoint_hint: "Use Ollama's local HTTP endpoint.".to_string(),
            model: "Model".to_string(),
            model_hint: "Select a model installed in Ollama before sending requests.".to_string(),
            model_required: "Select a model before using chat or autofix.".to_string(),
            timeout_secs: "Timeout".to_string(),
            timeout_hint: "Seconds".to_string(),
            lightweight_hint: "Small local models are recommended for limited memory.".to_string(),
            capabilities_section: "Features".to_string(),
            chat_enabled: "Chat".to_string(),
            autofix_enabled: "Lint autofix".to_string(),
        }
    }
}
