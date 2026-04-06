use super::*;

pub(super) fn handle_open_explorer(app: &mut KatanaApp, path: std::path::PathBuf) {
    if app.state.workspace.data.is_some() {
        manage::save_workspace_state(app);
    }
    app.state.workspace.is_loading = true;
    app.state.layout.status_message = Some((
        crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().status.opened_workspace,
            &[("name", "...")],
        ),
        crate::app_state::StatusType::Info,
    ));
    let (tx, rx) = std::sync::mpsc::channel();
    app.explorer_rx = Some(rx);
    let path_clone = path.clone();
    if let Some(token) = &app.state.workspace.cancel_token {
        token.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    app.state.workspace.cancel_token = Some(new_token.clone());
    let settings = app.state.config.settings.settings().workspace.clone();
    let in_memory_dirs = app.state.workspace.in_memory_dirs.clone();
    std::thread::spawn(move || {
        let fs = katana_platform::FilesystemService::new();
        let result = fs.open_workspace(
            &path_clone,
            &settings.ignored_directories,
            settings.max_depth,
            &settings.visible_extensions,
            &settings.extensionless_excludes,
            new_token,
            &in_memory_dirs,
        );
        let _ = tx.send((ExplorerLoadType::Open, path_clone, result));
    });
}

pub(super) fn finish_open_explorer(
    app: &mut KatanaApp,
    _path: std::path::PathBuf,
    ws: katana_core::workspace::Workspace,
) {
    let name = ws.name().unwrap_or("unknown").to_string();
    app.state.layout.status_message = Some((
        crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().status.opened_workspace,
            &[("name", name.as_str())],
        ),
        crate::app_state::StatusType::Success,
    ));
    app.state.workspace.data = Some(ws);
    app.state.document.open_documents.clear();
    app.state.document.active_doc_idx = None;
    app.state.document.tab_groups.clear();
    app.state.document.tab_view_modes.clear();
    app.state.document.tab_split_states.clear();
    app.state.document.recently_closed_tabs.clear();
    app.state.search.filter_cache = None;
    let path_str = app
        .state
        .workspace
        .data
        .as_ref()
        .unwrap()
        .root
        .display()
        .to_string();
    let (to_open, active_idx) = restore_session_tabs(app, &path_str);
    apply_session_tabs(app, to_open, active_idx, path_str);
}

fn restore_session_tabs(
    app: &mut KatanaApp,
    path_str: &str,
) -> (Vec<(String, bool)>, Option<usize>) {
    let workspace_root = app.state.workspace.data.as_ref().unwrap().root.clone();
    let cache_key = katana_platform::cache::PersistentKey::WorkspaceTabs {
        workspace_path: workspace_root.clone(),
    }
    .to_raw_key()
    .unwrap_or_default();

    let state_key = manage::compute_workspace_hash(&workspace_root.to_string_lossy());
    let restore_session = app
        .state
        .config
        .settings
        .settings()
        .workspace
        .restore_session;
    if !restore_session {
        return (vec![], None);
    }

    let mut state_json_opt = app.state.config.settings.load_workspace_state(&state_key);
    if state_json_opt.is_none()
        && let Some(cached_json) = app.state.config.cache.get_persistent(&cache_key)
    {
        let _ = app
            .state
            .config
            .settings
            .save_workspace_state(&state_key, &cached_json);
        state_json_opt = Some(cached_json);
    }

    if let Some(cache_json) = state_json_opt {
        parse_session_json(app, &cache_json)
    } else {
        let settings = app.state.config.settings.settings_mut();
        let is_same = settings.workspace.last_workspace.as_deref() == Some(path_str);
        if is_same {
            let tabs = settings
                .workspace
                .open_tabs
                .clone()
                .into_iter()
                .map(|t| (t, false))
                .collect();
            let idx = settings.workspace.active_tab_idx;
            (tabs, idx)
        } else {
            (vec![], None)
        }
    }
}

fn parse_session_json(
    app: &mut KatanaApp,
    cache_json: &str,
) -> (Vec<(String, bool)>, Option<usize>) {
    match serde_json::from_str::<WorkspaceTabSessionV2>(cache_json) {
        Ok(v2) => {
            let active_path = v2.active_path.clone();
            let to_open: Vec<(String, bool)> =
                v2.tabs.into_iter().map(|t| (t.path, t.pinned)).collect();
            let active_idx = active_path
                .as_ref()
                .and_then(|ap| to_open.iter().position(|(p, _)| p == ap));
            app.state.workspace.expanded_directories = v2
                .expanded_directories
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect();
            app.state.document.tab_groups = v2.groups;
            (to_open, active_idx)
        }
        Err(_) => parse_legacy_session(app, cache_json),
    }
}

fn parse_legacy_session(
    app: &mut KatanaApp,
    cache_json: &str,
) -> (Vec<(String, bool)>, Option<usize>) {
    #[derive(serde::Deserialize)]
    struct LegacyTabState {
        tabs: Vec<String>,
        active_idx: Option<usize>,
        #[serde(default)]
        expanded_directories: std::collections::HashSet<String>,
    }
    match serde_json::from_str::<LegacyTabState>(cache_json) {
        Ok(state) => {
            let to_open = state.tabs.into_iter().map(|t| (t, false)).collect();
            app.state.workspace.expanded_directories = state
                .expanded_directories
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect();
            (to_open, state.active_idx)
        }
        Err(e) => {
            tracing::warn!("Failed to deserialize tab state: {}", e);
            (vec![], None)
        }
    }
}

fn apply_session_tabs(
    app: &mut KatanaApp,
    mut to_open: Vec<(String, bool)>,
    active_idx: Option<usize>,
    path_str: String,
) {
    {
        let settings = app.state.config.settings.settings_mut();
        settings.workspace.persisted.retain(|p| p != &path_str);
        settings.workspace.persisted.push(path_str.clone());
        if !settings.workspace.histories.contains(&path_str) {
            settings.workspace.histories.push(path_str.clone());
        }
        settings.workspace.last_workspace = Some(path_str);
    }
    to_open.retain(|(p, _)| std::path::Path::new(p).exists());
    let existing: std::collections::HashSet<String> =
        to_open.iter().map(|(p, _)| p.clone()).collect();
    for g in &mut app.state.document.tab_groups {
        g.members.retain(|m| existing.contains(m));
    }
    app.state
        .document
        .tab_groups
        .retain(|g| !g.members.is_empty());
    if to_open.is_empty() {
        if !app.state.config.try_save_settings() {
            tracing::warn!("Failed to save settings");
        }
        return;
    }
    let active_idx_val = active_idx.unwrap_or(0).min(to_open.len().saturating_sub(1));
    for (i, (p, pinned)) in to_open.iter().enumerate() {
        let path = std::path::PathBuf::from(p);
        if i == active_idx_val {
            match app.fs.load_document(path) {
                Ok(mut doc) => {
                    doc.is_pinned = *pinned;
                    app.state.document.open_documents.push(doc);
                }
                Err(e) => tracing::error!("Failed to load document: {}", e),
            }
        } else {
            let mut doc = katana_core::document::Document::new_empty(path);
            doc.is_pinned = *pinned;
            app.state.document.open_documents.push(doc);
        }
        app.state
            .initialize_tab_split_state(std::path::PathBuf::from(p));
    }
    if !app.state.document.open_documents.is_empty() {
        let idx = active_idx
            .unwrap_or(0)
            .min(app.state.document.open_documents.len() - 1);
        app.state.document.active_doc_idx = Some(idx);
        let src = app.state.document.open_documents[idx].buffer.clone();
        let doc_path = app.state.document.open_documents[idx].path.clone();
        let concurrency = app
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        app.full_refresh_preview(&doc_path, &src, false, concurrency);
    }
    if !app.state.config.try_save_settings() {
        tracing::warn!("Failed to save settings");
    }
}
