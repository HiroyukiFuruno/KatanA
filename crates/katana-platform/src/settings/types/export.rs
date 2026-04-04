use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_pdf_engine")]
    pub pdf_engine: String,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_html_template")]
    pub html_template: String,
}
