use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsBehaviorMessages {
    pub section_title: String, pub confirm_close_dirty_tab: String,
    pub confirm_file_move: String,
    pub scroll_sync: String, pub auto_save: String,
    pub auto_save_interval: String, pub auto_save_interval_hint: String,
    pub auto_refresh: String, pub auto_refresh_interval: String,
    pub seconds: String, pub close_confirm_title: String,
    pub close_confirm_msg: String, pub close_confirm_discard: String,
    pub close_confirm_cancel: String, pub clear_http_cache: String,
    pub cache_retention_days: String, pub days_suffix: String,
    #[serde(default = "default_startup_behavior_section_title")]
    pub startup_behavior_section_title: String,
    pub toc_default_visible: String, pub explorer_default_visible: String,
    pub ingest_section_title: String, pub ingest_image_save_directory: String,
    pub ingest_image_name_format: String, pub ingest_create_directory: String,
    #[serde(default = "default_diagram_rendering_section_title")]
    pub diagram_rendering_section_title: String,
    #[serde(default = "default_diagram_concurrency")]
    pub diagram_concurrency: String,
    #[serde(default = "default_diagram_concurrency_hint")]
    pub diagram_concurrency_hint: String,
    #[serde(default = "default_diagram_concurrency_unlimited")]
    pub diagram_concurrency_unlimited: String,
    #[serde(default = "default_diagram_concurrency_unlimited_warning_title")]
    pub diagram_concurrency_unlimited_warning_title: String,
    #[serde(default = "default_diagram_concurrency_unlimited_warning_message")]
    pub diagram_concurrency_unlimited_warning_message: String,
}

fn default_startup_behavior_section_title() -> String {
    "Startup Behavior".to_string()
}

fn default_diagram_rendering_section_title() -> String {
    "Diagram Rendering".to_string()
}

fn default_diagram_concurrency() -> String {
    "Diagram render concurrency".to_string()
}

fn default_diagram_concurrency_hint() -> String {
    "If Markdown files with many diagrams feel heavy, lower this concurrency setting.".to_string()
}

fn default_diagram_concurrency_unlimited() -> String {
    "Unlimited diagram concurrency".to_string()
}

fn default_diagram_concurrency_unlimited_warning_title() -> String {
    "Enable unlimited diagram concurrency?".to_string()
}

fn default_diagram_concurrency_unlimited_warning_message() -> String {
    "This renders every detected diagram at the same time and may make the app or system unstable."
        .to_string()
}
