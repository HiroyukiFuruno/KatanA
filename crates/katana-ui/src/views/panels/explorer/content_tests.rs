#[cfg(test)]
mod tests {
    use crate::app_state::AppAction;
    use crate::views::panels::explorer::content::ExplorerContent;
    use std::path::Path;

    #[test]
    fn workspace_root_context_actions_target_workspace_root() {
        let workspace_root = Path::new("/workspace");

        assert_eq!(
            ExplorerContent::format_workspace_markdown_action(workspace_root),
            AppAction::FormatWorkspaceMarkdown(workspace_root.to_path_buf())
        );
        assert_eq!(
            ExplorerContent::new_workspace_root_file_action(workspace_root),
            AppAction::RequestNewFile(workspace_root.to_path_buf())
        );
        assert_eq!(
            ExplorerContent::new_workspace_root_directory_action(workspace_root),
            AppAction::RequestNewDirectory(workspace_root.to_path_buf())
        );
    }
}
