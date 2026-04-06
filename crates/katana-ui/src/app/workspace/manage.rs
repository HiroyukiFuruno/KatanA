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

pub(super) fn handle_remove_workspace(app: &mut KatanaApp, path: String) {
    let settings = app.state.config.settings.settings_mut();
    settings.workspace.paths.retain(|p| p != &path);
    if settings.workspace.last_workspace.as_deref() == Some(path.as_str()) {
        settings.workspace.last_workspace = None;
    }
    if !app.state.config.try_save_settings() {
        tracing::warn!("Failed to save settings after removing workspace");
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
