use super::types::*;
use crate::theme::ThemePreset;

pub struct SettingsDefaultOps;

pub(crate) const DEFAULT_FONT_SIZE: f32 = 14.0;
pub(crate) const DEFAULT_AUTO_SAVE_INTERVAL_SECS: f64 = 5.0;
pub(crate) const DEFAULT_AUTO_REFRESH_INTERVAL_SECS: f64 = 2.0;
pub const DEFAULT_MAX_DEPTH: usize = 10;
pub const DEFAULT_CACHE_RETENTION_DAYS: u32 = 7;
pub const DEFAULT_DIAGRAM_CONCURRENCY: usize = 4;
pub const DEFAULT_IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    ".terraform",
    "node_modules",
    "target",
    ".idea",
    ".vscode",
];

impl SettingsDefaultOps {
    pub fn default_version() -> String {
        "0.2.1".to_string()
    }
    pub fn default_language() -> String {
        "en".to_string()
    }
    pub fn default_theme() -> String {
        if crate::os_theme::OsThemeOps::is_dark_mode().unwrap_or(true) {
            "dark".to_string()
        } else {
            "light".to_string()
        }
    }
    pub fn default_ui_contrast_offset() -> f32 {
        0.0
    }
    pub fn default_font_size() -> f32 {
        DEFAULT_FONT_SIZE
    }
    pub fn default_font_family() -> String {
        "monospace".to_string()
    }
    pub fn default_auto_save_interval_secs() -> f64 {
        DEFAULT_AUTO_SAVE_INTERVAL_SECS
    }
    pub fn default_auto_refresh_interval_secs() -> f64 {
        DEFAULT_AUTO_REFRESH_INTERVAL_SECS
    }
    pub fn default_ignored_directories() -> Vec<String> {
        DEFAULT_IGNORED_DIRECTORIES
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    pub fn default_max_depth() -> usize {
        DEFAULT_MAX_DEPTH
    }
    pub fn default_visible_extensions() -> Vec<String> {
        vec!["md".to_string(), "markdown".to_string(), "txt".to_string()]
    }
    pub fn default_extensionless_excludes() -> Vec<String> {
        vec!["LICENSE".to_string(), "Makefile".to_string()]
    }
    pub fn default_restore_session() -> bool {
        true
    }
    pub fn default_cache_retention() -> u32 {
        DEFAULT_CACHE_RETENTION_DAYS
    }
    pub fn default_diagram_concurrency() -> usize {
        DEFAULT_DIAGRAM_CONCURRENCY
    }

    pub fn select_initial_preset() -> ThemePreset {
        if crate::os_theme::OsThemeOps::is_dark_mode().unwrap_or(true) {
            ThemePreset::KatanaDark
        } else {
            ThemePreset::KatanaLight
        }
    }
    pub fn select_preset_for_mode(is_dark: Option<bool>) -> ThemePreset {
        match is_dark {
            Some(true) => ThemePreset::KatanaDark,
            Some(false) => ThemePreset::KatanaLight,
            None => ThemePreset::KatanaDark,
        }
    }

    pub fn true_default() -> bool {
        true
    }

    pub fn false_default() -> bool {
        false
    }

    pub fn default_pdf_engine() -> String {
        "builtin".to_string()
    }

    pub fn default_html_template() -> String {
        "default".to_string()
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SettingsDefaultOps::default_version(),
            theme: ThemeSettings::default(),
            font: FontSettings::default(),
            layout: LayoutSettings::default(),
            workspace: WorkspaceSettings::default(),
            search: SearchSettings::default(),
            performance: PerformanceSettings::default(),
            export: ExportSettings::default(),
            updates: UpdateSettings::default(),
            behavior: BehaviorSettings::default(),
            terms_accepted_version: None,
            language: SettingsDefaultOps::default_language(),
            extra: Vec::new(),
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            theme: SettingsDefaultOps::default_theme(),
            ui_contrast_offset: SettingsDefaultOps::default_ui_contrast_offset(),
            preset: ThemePreset::default(),
            custom_color_overrides: None,
            custom_themes: Vec::new(),
            active_custom_theme: None,
        }
    }
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            size: SettingsDefaultOps::default_font_size(),
            family: SettingsDefaultOps::default_font_family(),
        }
    }
}
