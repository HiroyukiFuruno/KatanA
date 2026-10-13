use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiffViewMode {
    #[default]
    Split,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub auto_save: bool,
    #[serde(
        default = "super::super::defaults::SettingsDefaultOps::default_auto_save_interval_secs"
    )]
    pub auto_save_interval_secs: f64,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub auto_refresh: bool,
    #[serde(
        default = "super::super::defaults::SettingsDefaultOps::default_auto_refresh_interval_secs"
    )]
    pub auto_refresh_interval_secs: f64,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub scroll_sync_enabled: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub confirm_close_dirty_tab: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub confirm_file_move: bool,
    #[serde(default)]
    pub diff_view_mode: DiffViewMode,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub slideshow_hover_highlight: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub slideshow_show_diagram_controls: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityRailItem {
    AddWorkspace,
    WorkspaceToggle,
    ExplorerToggle,
    Search,
    History,
}
