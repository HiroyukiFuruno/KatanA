use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdateMessages {
    pub section_title: String,
    pub interval: String,
    pub never: String,
    pub daily: String,
    pub weekly: String,
    pub monthly: String,
    pub check_now: String,
    pub plantuml_section_title: String,
    pub plantuml_installed: String,
    pub plantuml_not_installed: String,
    pub plantuml_update_now: String,
    #[serde(default)]
    pub drawio_section_title: String,
    #[serde(default)]
    pub drawio_installed: String,
    #[serde(default)]
    pub drawio_not_installed: String,
    #[serde(default)]
    pub drawio_update_now: String,
    #[serde(default = "default_mermaid_section_title")]
    pub mermaid_section_title: String,
    #[serde(default = "default_mermaid_installed")]
    pub mermaid_installed: String,
    #[serde(default = "default_mermaid_not_installed")]
    pub mermaid_not_installed: String,
    #[serde(default = "default_mermaid_update_now")]
    pub mermaid_update_now: String,
}

fn default_mermaid_section_title() -> String {
    "Mermaid".to_string()
}

fn default_mermaid_installed() -> String {
    "Mermaid.js is installed at {path}".to_string()
}

fn default_mermaid_not_installed() -> String {
    "Mermaid.js is not installed.".to_string()
}

fn default_mermaid_update_now() -> String {
    "Update Mermaid.js to Latest".to_string()
}
