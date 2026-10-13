use crate::shell::KatanaApp;

pub(crate) struct WorkspaceRegistrationOps;

impl WorkspaceRegistrationOps {
    pub(crate) fn sync_opened_workspace(app: &mut KatanaApp, path_str: &str) {
        if app
            .state
            .workspace
            .is_temporary_root(std::path::Path::new(path_str))
        {
            Self::remove_temporary_workspace_refs(app, path_str);
        } else {
            Self::remember_workspace(app, path_str);
        }
        let _ = app.state.global_workspace.save();
    }

    fn remove_temporary_workspace_refs(app: &mut KatanaApp, path_str: &str) {
        let global_state = app.state.global_workspace.state_mut();
        global_state.persisted.retain(|path| path != path_str);
        global_state.histories.retain(|path| path != path_str);
        global_state
            .open_workspace_tabs
            .retain(|path| path != path_str);
        if global_state.active_workspace.as_deref() == Some(path_str) {
            global_state.active_workspace = global_state.open_workspace_tabs.first().cloned();
        }
        let settings = app.state.config.settings.settings_mut();
        if settings.workspace.last_workspace.as_deref() == Some(path_str) {
            settings.workspace.last_workspace = None;
        }
    }

    fn remember_workspace(app: &mut KatanaApp, path_str: &str) {
        let open_workspace_in_tabs = app
            .state
            .config
            .settings
            .settings()
            .workspace
            .open_workspace_in_tabs;
        let global_state = app.state.global_workspace.state_mut();
        /* WHY: All open-workspace routes (dialog, list, history) share identical behavior:
         * both `persisted` and `histories` are updated here. */
        global_state.persisted.retain(|path| path != path_str);
        global_state.persisted.push(path_str.to_string());
        global_state.histories.retain(|path| path != path_str);
        global_state.histories.push(path_str.to_string());
        let tabs = crate::app::workspace::tabs::WorkspaceTabsOps::resolve_open_tabs_after_open(
            global_state.open_workspace_tabs.clone(),
            global_state.active_workspace.clone(),
            path_str,
            open_workspace_in_tabs,
        );
        global_state.open_workspace_tabs = tabs.open_tabs;
        global_state.active_workspace = tabs.active_workspace;
        app.state.workspace.scroll_to_workspace_tab = Some(std::path::PathBuf::from(path_str));

        let settings = app.state.config.settings.settings_mut();
        settings.workspace.last_workspace = Some(path_str.to_string());
    }
}
