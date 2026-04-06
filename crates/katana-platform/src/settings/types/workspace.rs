use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    #[serde(default)]
    pub last_workspace: Option<String>,
    #[serde(default)]
    pub persisted: Vec<String>,
    #[serde(default)]
    pub histories: Vec<String>,
    #[serde(default)]
    pub open_tabs: Vec<String>,
    #[serde(default)]
    pub active_tab_idx: Option<usize>,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_ignored_directories")]
    pub ignored_directories: Vec<String>,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_visible_extensions")]
    pub visible_extensions: Vec<String>,
    #[serde(
        default = "super::super::defaults::SettingsDefaultOps::default_extensionless_excludes"
    )]
    pub extensionless_excludes: Vec<String>,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_restore_session")]
    pub restore_session: bool,
}
