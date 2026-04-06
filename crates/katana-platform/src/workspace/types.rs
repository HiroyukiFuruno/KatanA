use serde::{Deserialize, Serialize};

/// Global workspace state representing the user's registered paths and history.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalWorkspaceState {
    /// Workspaces explicitly registered/persisted by the user.
    #[serde(default)]
    pub persisted: Vec<String>,

    /// Recently opened workspaces (history).
    #[serde(default)]
    pub histories: Vec<String>,
}
