use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MenuMessages {
    pub file: String,
    pub settings: String,
    pub language: String,
    pub open_workspace: String,
    pub save: String,
    pub open_all: String,
    pub about: String,
    pub quit: String,
    pub hide: String,
    pub hide_others: String,
    pub show_all: String,
    #[serde(default)]
    pub export: String,
    #[serde(default)]
    pub export_html: String,
    #[serde(default)]
    pub export_pdf: String,
    #[serde(default)]
    pub export_png: String,
    #[serde(default)]
    pub export_jpg: String,
    pub help: String,
    pub check_updates: String,
    #[serde(default = "default_menu_release_notes")]
    pub release_notes: String,
    #[serde(default = "default_menu_command_palette")]
    pub command_palette: String,
    #[serde(default = "default_menu_view")]
    pub view: String,
    #[serde(default = "default_menu_close_workspace")]
    pub close_workspace: String,
    #[serde(default = "default_menu_demo")]
    pub demo: String,

    #[serde(default = "default_menu_lang_en")]
    pub language_en: String,
    #[serde(default = "default_menu_lang_ja")]
    pub language_ja: String,
    #[serde(default = "default_menu_lang_zh_cn")]
    pub language_zh_cn: String,
    #[serde(default = "default_menu_lang_zh_tw")]
    pub language_zh_tw: String,
    #[serde(default = "default_menu_lang_ko")]
    pub language_ko: String,
    #[serde(default = "default_menu_lang_pt")]
    pub language_pt: String,
    #[serde(default = "default_menu_lang_fr")]
    pub language_fr: String,
    #[serde(default = "default_menu_lang_de")]
    pub language_de: String,
    #[serde(default = "default_menu_lang_es")]
    pub language_es: String,
    #[serde(default = "default_menu_lang_it")]
    pub language_it: String,
}

pub(super) fn default_menu_lang_en() -> String {
    "English".to_string()
}
pub(super) fn default_menu_lang_ja() -> String {
    "日本語".to_string()
}
pub(super) fn default_menu_lang_zh_cn() -> String {
    "中文 (简体)".to_string()
}
pub(super) fn default_menu_lang_zh_tw() -> String {
    "中文 (繁體)".to_string()
}
pub(super) fn default_menu_lang_ko() -> String {
    "한국어".to_string()
}
pub(super) fn default_menu_lang_pt() -> String {
    "Português".to_string()
}
pub(super) fn default_menu_lang_fr() -> String {
    "Français".to_string()
}
pub(super) fn default_menu_lang_de() -> String {
    "Deutsch".to_string()
}
pub(super) fn default_menu_lang_es() -> String {
    "Español".to_string()
}
pub(super) fn default_menu_lang_it() -> String {
    "Italiano".to_string()
}

pub(super) fn default_menu_close_workspace() -> String {
    "Close Workspace".to_string()
}

pub(super) fn default_menu_demo() -> String {
    "Demo".to_string()
}

pub(super) fn default_menu_command_palette() -> String {
    "Command Palette…".to_string()
}

pub(super) fn default_menu_view() -> String {
    "View".to_string()
}

pub(super) fn default_menu_release_notes() -> String {
    "Release Notes".to_string()
}
