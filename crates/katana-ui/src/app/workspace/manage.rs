use super::*;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub(super) fn compute_workspace_hash(path_str: &str) -> String {
    let mut s = path_str.to_string();
    if s.ends_with('/') || s.ends_with('\\') {
        s.pop();
    }
    let mut hash: u64 = FNV_OFFSET_BASIS;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:x}", hash)
}

pub(super) fn remove_workspace(app: &mut KatanaApp, path: &str) {
    let global_state = app.state.global_workspace.state_mut();
    global_state.persisted.retain(|p| p != path);
    global_state.histories.retain(|p| p != path);
    let _ = app.state.global_workspace.save();

    let settings = app.state.config.settings.settings_mut();

    /* WHY: Check if the current workspace was removed */
    if let Some(ref current) = settings.workspace.last_workspace
        && current == path
    {
        /* WHY: Unset the last workspace since it was removed */
        settings.workspace.last_workspace = None;
        /* WHY: Also reset tree structures since we have no open workspace */
        app.state.workspace.data = None;
    }

    if let Err(e) = app.state.config.settings.save() {
        tracing::error!("Failed to save settings: {}", e);
    }
}

pub(super) fn handle_remove_explorer(app: &mut KatanaApp, path: String) {
    let no_persisted_left = {
        let global_state = app.state.global_workspace.state_mut();
        /* WHY: Only deregister from `persisted`. `histories` is independent and
         * stays intact so the user can still re-open from Recent Workspaces. */
        global_state.persisted.retain(|p| p != &path);
        global_state.persisted.is_empty()
    };
    let _ = app.state.global_workspace.save();

    let settings = app.state.config.settings.settings_mut();
    if settings.workspace.last_workspace.as_deref() == Some(path.as_str()) {
        settings.workspace.last_workspace = None;
    }

    /* WHY: If the deregistered workspace is currently open, or no persisted entries
     * remain, close the workspace to return to the initial screen. */
    let is_current_open = app
        .state
        .workspace
        .data
        .as_ref()
        .is_some_and(|ws| ws.root.to_string_lossy() == path);
    if is_current_open || no_persisted_left {
        app.pending_action = crate::app_state::AppAction::CloseWorkspace;
    }

    if !app.state.config.try_save_settings() {
        tracing::warn!("Failed to save settings after removing workspace");
        app.state.layout.status_message = Some((
            crate::i18n::I18nOps::get()
                .status
                .error_save_settings
                .clone(),
            crate::app_state::StatusType::Error,
        ));
    } else {
        app.state.layout.status_message = Some((
            crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().status.removed_workspace,
                &[("path", path.as_str())],
            ),
            crate::app_state::StatusType::Success,
        ));
    }
}

pub(super) fn handle_remove_workspace_history(app: &mut KatanaApp, path: String) {
    /* WHY: Only deregister from `histories`. `persisted` is not affected. */
    let global_state = app.state.global_workspace.state_mut();
    global_state.histories.retain(|p| p != &path);
    if let Err(e) = app.state.global_workspace.save() {
        tracing::warn!(
            "Failed to save global workspace state after deregistering history: {}",
            e
        );
    }
}

pub(super) fn save_workspace_state(app: &mut KatanaApp) {
    let idx = app.state.document.active_doc_idx;
    let open_tabs: Vec<String> = app
        .state
        .document
        .open_documents
        .iter()
        .map(|d| d.path.display().to_string())
        .filter(|p| !p.starts_with("Katana://"))
        .collect();
    let settings = app.state.config.settings.settings_mut();
    settings.workspace.open_tabs = open_tabs.clone();
    settings.workspace.active_tab_idx = idx;
    if !app.state.config.try_save_settings() {
        tracing::warn!("Failed to save workspace tab state");
    }
    let Some(ws) = &app.state.workspace.data else {
        return;
    };
    if app.state.workspace.is_temporary_root(&ws.root) {
        return;
    }
    let state_key = compute_workspace_hash(&ws.root.to_string_lossy());
    let expanded: std::collections::HashSet<String> = app
        .state
        .workspace
        .expanded_directories
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let tabs_v2: Vec<WorkspaceTabEntry> = app
        .state
        .document
        .open_documents
        .iter()
        .filter(|d| !d.path.display().to_string().starts_with("Katana://"))
        .map(|d| WorkspaceTabEntry {
            path: d.path.display().to_string(),
            pinned: d.is_pinned,
        })
        .collect();
    let active_path = app
        .state
        .document
        .active_document()
        .map(|d| d.path.display().to_string());
    let state = WorkspaceTabSessionV2 {
        version: 2,
        tabs: tabs_v2,
        active_path,
        expanded_directories: expanded,
        groups: app.state.document.tab_groups.clone(),
    };
    match serde_json::to_string(&state) {
        Ok(json) => {
            if let Err(e) = app
                .state
                .config
                .settings
                .save_workspace_state(&state_key, &json)
            {
                tracing::warn!("Failed to save workspace state: {}", e);
            }
        }
        Err(e) => tracing::warn!("Failed to serialize tab state: {}", e),
    }
}
