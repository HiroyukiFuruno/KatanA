use super::KatanaApp;

impl KatanaApp {
    pub(crate) fn clear_transient_workspace_restore_state(&mut self) {
        let changed_global = {
            let global_state = self.state.global_workspace.state_mut();
            let persisted_len = global_state.persisted.len();
            let histories_len = global_state.histories.len();
            global_state
                .persisted
                .retain(|path| !Self::is_transient_workspace_path(path));
            global_state
                .histories
                .retain(|path| !Self::is_transient_workspace_path(path));
            persisted_len != global_state.persisted.len()
                || histories_len != global_state.histories.len()
        };
        if changed_global {
            let _ = self.state.global_workspace.save();
        }

        let settings = self.state.config.settings.settings_mut();
        let should_clear_last = settings
            .workspace
            .last_workspace
            .as_ref()
            .is_some_and(|path| Self::is_transient_workspace_path(path));
        if should_clear_last {
            settings.workspace.last_workspace = None;
            settings.workspace.open_tabs.clear();
            settings.workspace.active_tab_idx = None;
            if !self.state.config.try_save_settings() {
                tracing::warn!("Failed to clear transient workspace restore state");
            }
        }
    }

    fn is_transient_workspace_path(path: &str) -> bool {
        let workspace_path = std::path::Path::new(path);
        workspace_path.starts_with(std::env::temp_dir())
            || path.contains("/var/folders/")
            || path.contains("\\Temp\\")
    }
}
