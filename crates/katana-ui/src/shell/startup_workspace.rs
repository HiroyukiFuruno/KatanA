use crate::app_state::AppAction;

pub(super) struct WorkspaceStartupOps;

impl WorkspaceStartupOps {
    pub(super) fn restore_workspace(app: &mut super::types::KatanaApp) {
        let (last_workspace, should_save_global_workspace) = Self::resolve_restore_workspace(app);
        if should_save_global_workspace && let Err(e) = app.state.global_workspace.save() {
            tracing::warn!("Failed to save pruned workspace tabs: {}", e);
        }
        if let Some(workspace_path) = last_workspace {
            if std::path::Path::new(&workspace_path).exists() {
                app.pending_action =
                    AppAction::OpenWorkspace(std::path::PathBuf::from(workspace_path));
            } else {
                tracing::warn!(
                    "Saved workspace path no longer exists, skipping restore: {}",
                    workspace_path
                );
            }
        }
    }

    fn resolve_restore_workspace(app: &mut super::types::KatanaApp) -> (Option<String>, bool) {
        let global_state = app.state.global_workspace.state_mut();
        let tab_count_before_prune = global_state.open_workspace_tabs.len();
        global_state
            .open_workspace_tabs
            .retain(|path| std::path::Path::new(path).exists());
        let pruned_missing_tabs = tab_count_before_prune != global_state.open_workspace_tabs.len();
        let active_workspace = global_state
            .active_workspace
            .clone()
            .filter(|path| global_state.open_workspace_tabs.contains(path));
        let active_workspace_changed = global_state.active_workspace != active_workspace;
        global_state.active_workspace = active_workspace.clone();
        let restore_workspace = active_workspace
            .or_else(|| global_state.open_workspace_tabs.first().cloned())
            .or_else(|| {
                app.state
                    .config
                    .settings
                    .settings()
                    .workspace
                    .last_workspace
                    .clone()
            });
        (
            restore_workspace,
            pruned_missing_tabs || active_workspace_changed,
        )
    }
}
