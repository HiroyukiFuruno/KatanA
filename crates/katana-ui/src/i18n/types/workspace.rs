use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMessages {
    pub no_workspace_open: String,
    pub no_document_selected: String,
    pub explorer_title: String,
    pub workspace_history_title: String,
    pub recent_workspaces: String,
    pub sidebar_workspace_tooltip: String,
    pub sidebar_history_tooltip: String,
    pub no_recent_workspaces: String,
    pub no_saved_workspaces: String,
    pub metadata_tooltip: String,
    pub path_label: String,
    pub flat_view: String,
    pub filter_regex_hint: String,
    pub open_folder_hint: String,
    pub open_workspace_button: String,
    pub remove_history_tooltip: String,
    pub max_depth_exceeded: String,
}
