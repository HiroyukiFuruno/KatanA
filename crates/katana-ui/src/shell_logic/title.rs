use super::ShellLogicOps;
use std::path::Path;

impl ShellLogicOps {
    pub fn format_window_title(fname: &str, rel: &str, release_notes: &str) -> String {
        Self::format_document_title(fname, rel, release_notes)
    }

    pub fn format_document_title(fname: &str, rel: &str, release_notes: &str) -> String {
        if fname == release_notes {
            return format!("{release_notes} - KatanA");
        }
        format!("{fname} ({rel}) - KatanA")
    }

    pub fn format_workspace_window_title(ws_root: Option<&Path>) -> String {
        let Some(root) = ws_root else {
            return "KatanA".to_string();
        };
        let workspace_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("KatanA");
        format!("{workspace_name} - KatanA")
    }
}
