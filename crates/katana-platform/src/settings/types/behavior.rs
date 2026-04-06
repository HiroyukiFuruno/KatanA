use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityRailItem {
    AddWorkspace,
    WorkspaceToggle,
    ExplorerToggle,
    Search,
    History,
}
