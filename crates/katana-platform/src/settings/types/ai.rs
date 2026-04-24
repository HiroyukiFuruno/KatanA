use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSettings {
    #[serde(default)]
    pub ollama: OllamaSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OllamaSettings {
    #[serde(default = "default_ollama_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub selected_model: String,
    #[serde(default = "default_ollama_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub chat_enabled: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub autofix_enabled: bool,
}

impl Default for OllamaSettings {
    fn default() -> Self {
        Self {
            endpoint: default_ollama_endpoint(),
            selected_model: String::new(),
            timeout_secs: default_ollama_timeout_secs(),
            chat_enabled: true,
            autofix_enabled: true,
        }
    }
}

fn default_ollama_endpoint() -> String {
    katana_core::ai::DEFAULT_OLLAMA_ENDPOINT.to_string()
}

fn default_ollama_timeout_secs() -> u64 {
    katana_core::ai::DEFAULT_OLLAMA_TIMEOUT_SECS
}
