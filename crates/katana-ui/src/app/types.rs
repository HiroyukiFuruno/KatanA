use crate::state::document::TabGroup;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WorkspaceTabEntry {
    pub path: String,
    pub pinned: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WorkspaceTabSessionV2 {
    pub version: u32,
    pub tabs: Vec<WorkspaceTabEntry>,
    pub active_path: Option<String>,
    #[serde(default)]
    pub expanded_directories: HashSet<String>,
    #[serde(default)]
    pub groups: Vec<TabGroup>,
}
