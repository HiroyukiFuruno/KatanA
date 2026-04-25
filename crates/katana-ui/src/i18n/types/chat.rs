use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessages {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_close")]
    pub close: String,
    #[serde(default = "default_pin")]
    pub pin: String,
    #[serde(default = "default_model_required")]
    pub model_required: String,
    #[serde(default = "default_disabled_error")]
    pub disabled_error: String,
    #[serde(default = "default_interrupted_error")]
    pub interrupted_error: String,
    #[serde(default = "default_open_settings")]
    pub open_settings: String,
    #[serde(default = "default_model_label")]
    pub model_label: String,
    #[serde(default = "default_empty_session")]
    pub empty_session: String,
    #[serde(default = "default_waiting")]
    pub waiting: String,
    #[serde(default = "default_input_hint")]
    pub input_hint: String,
    #[serde(default = "default_send")]
    pub send: String,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default = "default_assistant")]
    pub assistant: String,
}

impl Default for ChatMessages {
    fn default() -> Self {
        Self {
            title: default_title(),
            close: default_close(),
            pin: default_pin(),
            model_required: default_model_required(),
            disabled_error: default_disabled_error(),
            interrupted_error: default_interrupted_error(),
            open_settings: default_open_settings(),
            model_label: default_model_label(),
            empty_session: default_empty_session(),
            waiting: default_waiting(),
            input_hint: default_input_hint(),
            send: default_send(),
            user: default_user(),
            assistant: default_assistant(),
        }
    }
}

fn default_title() -> String {
    "Chat".to_string()
}

fn default_close() -> String {
    "Close chat".to_string()
}

fn default_pin() -> String {
    "Pin chat panel".to_string()
}

fn default_model_required() -> String {
    "Select an Ollama model in Settings.".to_string()
}

fn default_disabled_error() -> String {
    "Chat is disabled in AI settings.".to_string()
}

fn default_interrupted_error() -> String {
    "Chat request was interrupted.".to_string()
}

fn default_open_settings() -> String {
    "Open AI Settings".to_string()
}

fn default_model_label() -> String {
    "Model: {model}".to_string()
}

fn default_empty_session() -> String {
    "No messages in this app session.".to_string()
}

fn default_waiting() -> String {
    "Waiting for Ollama...".to_string()
}

fn default_input_hint() -> String {
    "Ask local LLM".to_string()
}

fn default_send() -> String {
    "Send".to_string()
}

fn default_user() -> String {
    "You".to_string()
}

fn default_assistant() -> String {
    "Assistant".to_string()
}
