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
        let settings = app.state.config.settings.settings_mut();
        if settings.workspace.last_workspace.as_deref() == Some(path_str) {
            settings.workspace.last_workspace = None;
        }
    }

    fn remember_workspace(app: &mut KatanaApp, path_str: &str) {
        let global_state = app.state.global_workspace.state_mut();
        /* WHY: All open-workspace routes (dialog, list, history) share identical behavior:
         * both `persisted` and `histories` are updated here. */
        global_state.persisted.retain(|path| path != path_str);
        global_state.persisted.push(path_str.to_string());
        global_state.histories.retain(|path| path != path_str);
        global_state.histories.push(path_str.to_string());

        let settings = app.state.config.settings.settings_mut();
        settings.workspace.last_workspace = Some(path_str.to_string());
    }
}
