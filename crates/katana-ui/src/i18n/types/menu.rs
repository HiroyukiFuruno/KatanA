use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuMessages {
    pub file: String,
    pub settings: String,
    pub language: String,
    pub open_workspace: String,
    pub close_workspace: String,
    pub save: String,
    pub open_all: String,
    pub export: String,
    pub export_html: String,
    pub export_pdf: String,
    pub export_png: String,
    pub export_jpg: String,
    pub about: String,
    pub quit: String,
    pub hide: String,
    pub hide_others: String,
    pub show_all: String,
    pub help: String,
    pub check_updates: String,
    pub release_notes: String,
    pub view: String,
    pub command_palette: String,
    pub language_en: String,
    pub language_ja: String,
    pub language_zh_cn: String,
    pub language_zh_tw: String,
    pub language_ko: String,
    pub language_pt: String,
    pub language_fr: String,
    pub language_de: String,
    pub language_es: String,
    pub language_it: String,
    #[serde(default = "default_menu_demo")]
    pub demo: String,
    #[serde(default = "default_menu_welcome")]
    pub welcome_screen: String,
    #[serde(default = "default_menu_guide")]
    pub user_guide: String,
    #[serde(default = "default_menu_github")]
    pub github: String,
}

fn default_menu_demo() -> String {
    "Demo".to_string()
}
fn default_menu_welcome() -> String {
    "Welcome Screen".to_string()
}
fn default_menu_guide() -> String {
    "User Guide".to_string()
}
fn default_menu_github() -> String {
    "GitHub Repository".to_string()
}
