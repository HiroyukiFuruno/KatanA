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
}
