use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestSettings {
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_image_save_directory")]
    pub image_save_directory: String,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub create_directory_if_not_exists: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_image_name_format")]
    pub image_name_format: String,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub show_confirmation_dialog: bool,
}
