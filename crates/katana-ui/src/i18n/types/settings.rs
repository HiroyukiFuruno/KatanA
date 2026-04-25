use super::settings_ai::SettingsAiMessages;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsMessages {
    pub title: String,
    pub toc_visible: String,
    pub theme: SettingsThemeMessages,
    pub font: SettingsFontMessages,
    pub layout: SettingsLayoutMessages,
    pub preview: SettingsPreviewMessages,
    pub color: SettingsColorMessages,
    pub workspace: SettingsWorkspaceMessages,
    pub icons: SettingsIconsMessages,
    pub tabs: Vec<SettingsTabItem>,
    pub updates: SettingsUpdateMessages,
    pub behavior: SettingsBehaviorMessages,
    pub shortcuts: SettingsShortcutsMessages,
    #[serde(default)]
    pub ai: SettingsAiMessages,
    pub general: String,
}

impl SettingsMessages {
    pub fn tab_name(&self, key: &str) -> String {
        self.tabs
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| Self::default_tab_name(key))
    }

    fn default_tab_name(key: &str) -> String {
        match key {
            "ai" => "AI".to_string(),
            _ => key.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsThemeMessages {
    pub preset: String,
    pub dark_section: String,
    pub light_section: String,
    pub icon_pack: String,
    pub custom_colors: String,
    pub reset_custom: String,
    pub custom_section: String,
    pub delete_custom: String,
    pub save_custom_theme: String,
    pub save_custom_theme_title: String,
    pub theme_name_label: String,
    pub duplicate: String,
    pub reset_contrast: String,
    pub ui_contrast_offset: String,
    pub show_more: String,
    pub show_less: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsFontMessages {
    pub size: String,
    pub family: String,
    pub size_slider_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsLayoutMessages {
    pub split_direction: String,
    pub horizontal: String,
    pub vertical: String,
    pub pane_order: String,
    pub editor_first: String,
    pub preview_first: String,
    pub toc_position: String,
    pub left: String,
    pub right: String,
    pub accordion_vertical_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPreviewMessages {
    pub title: String,
    pub heading: String,
    pub normal_text: String,
    pub accent_link: String,
    pub secondary_text: String,
    pub code_sample: String,
}

pub use crate::i18n::types::settings_color::SettingsColorMessages;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWorkspaceMessages {
    pub max_depth: String,
    pub ignored_directories: String,
    pub ignored_directories_hint: String,
    pub visible_extensions: String,
    pub no_extension_label: String,
    pub no_extension_warning_title: String,
    pub no_extension_warning: String,
    pub extensionless_excludes: String,
    pub extensionless_excludes_hint: String,
    pub enable_drawio_mount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsIconsMessages {
    pub preset_label: String,
    pub custom_preset: String,
    pub save_preset: String,
    pub revert_default: String,
    pub colorful_vendor_icons_label: String,
    pub table_header_icon: String,
    pub table_header_vendor: String,
    pub table_header_color: String,
    pub table_header_border: String,
    pub table_header_preview: String,
    pub preset_name: String,
    #[serde(default = "default_icons_search_placeholder")]
    pub search_placeholder: String,
}

fn default_icons_search_placeholder() -> String {
    "Search icons…".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsTabItem {
    pub key: String,
    pub name: String,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsBehaviorMessages {
    pub section_title: String,
    pub confirm_close_dirty_tab: String,
    pub scroll_sync: String,
    pub auto_save: String,
    pub auto_save_interval: String,
    pub auto_save_interval_hint: String,
    pub auto_refresh: String,
    pub auto_refresh_interval: String,
    pub seconds: String,
    pub close_confirm_title: String,
    pub close_confirm_msg: String,
    pub close_confirm_discard: String,
    pub close_confirm_cancel: String,
    pub clear_http_cache: String,
    pub cache_retention_days: String,
    pub days_suffix: String,
    pub toc_default_visible: String,
    pub explorer_default_visible: String,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsShortcutsMessages {
    pub edit: String, pub press_keys: String, pub restore_defaults: String, pub none: String,
    pub conflict_warning: String, pub capture_prompt: String, pub confirm_key: String,
    pub cancel_key: String, pub search_placeholder: String,
}
