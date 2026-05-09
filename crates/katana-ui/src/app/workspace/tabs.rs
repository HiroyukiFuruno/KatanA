#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceOpenTabsState {
    pub open_tabs: Vec<String>,
    pub active_workspace: Option<String>,
}

pub(crate) struct WorkspaceTabsOps;

impl WorkspaceTabsOps {
    pub(crate) fn resolve_open_tabs_after_open(
        mut open_tabs: Vec<String>,
        active_workspace: Option<String>,
        opening_workspace: &str,
        open_workspace_in_tabs: bool,
    ) -> WorkspaceOpenTabsState {
        let opening = opening_workspace.to_string();
        if open_tabs.iter().any(|path| path == &opening) {
            return WorkspaceOpenTabsState {
                open_tabs,
                active_workspace: Some(opening),
            };
        }
        if open_workspace_in_tabs {
            open_tabs.push(opening.clone());
            return WorkspaceOpenTabsState {
                open_tabs,
                active_workspace: Some(opening),
            };
        }
        let replace_index = active_workspace
            .and_then(|active| open_tabs.iter().position(|path| path == &active))
            .unwrap_or(0);
        if open_tabs.is_empty() {
            open_tabs.push(opening.clone());
        } else {
            open_tabs[replace_index] = opening.clone();
        }
        WorkspaceOpenTabsState {
            open_tabs,
            active_workspace: Some(opening),
        }
    }

    pub(crate) fn resolve_open_tabs_after_close(
        mut open_tabs: Vec<String>,
        active_workspace: Option<String>,
        closing_workspace: &str,
    ) -> WorkspaceOpenTabsState {
        let Some(closing_index) = open_tabs.iter().position(|path| path == closing_workspace)
        else {
            return WorkspaceOpenTabsState {
                open_tabs,
                active_workspace,
            };
        };
        open_tabs.remove(closing_index);
        if active_workspace.as_deref() != Some(closing_workspace) {
            return WorkspaceOpenTabsState {
                open_tabs,
                active_workspace,
            };
        }
        let next_active = if open_tabs.is_empty() {
            None
        } else {
            let next_index = closing_index.min(open_tabs.len() - 1);
            Some(open_tabs[next_index].clone())
        };
        WorkspaceOpenTabsState {
            open_tabs,
            active_workspace: next_active,
        }
    }

    pub(crate) fn resolve_open_tabs_after_reorder(
        mut open_tabs: Vec<String>,
        active_workspace: Option<String>,
        from: usize,
        to: usize,
    ) -> WorkspaceOpenTabsState {
        if from >= open_tabs.len() || to > open_tabs.len() || from == to || from + 1 == to {
            return WorkspaceOpenTabsState {
                open_tabs,
                active_workspace,
            };
        }
        let tab = open_tabs.remove(from);
        let actual_to = if to > from { to - 1 } else { to };
        open_tabs.insert(actual_to, tab);
        WorkspaceOpenTabsState {
            open_tabs,
            active_workspace,
        }
    }
}

pub(super) fn handle_select_workspace_tab(
    app: &mut crate::shell::KatanaApp,
    path: std::path::PathBuf,
) {
    app.state.workspace.scroll_to_workspace_tab = Some(path.clone());
    if app
        .state
        .workspace
        .data
        .as_ref()
        .is_some_and(|workspace| workspace.root == path)
    {
        let path_str = path.display().to_string();
        app.state.global_workspace.state_mut().active_workspace = Some(path_str);
        let _ = app.state.global_workspace.save();
        return;
    }
    super::manage::save_workspace_state(app);
    super::open::WorkspaceOpenHandlersOps::handle_open_explorer(app, path);
}

pub(super) fn handle_close_workspace_tab(
    app: &mut crate::shell::KatanaApp,
    path: std::path::PathBuf,
) {
    let path_str = path.display().to_string();
    let is_active = app
        .state
        .global_workspace
        .state()
        .active_workspace
        .as_deref()
        == Some(path_str.as_str());
    if is_active {
        super::manage::save_workspace_state(app);
    }
    let next_state = {
        let global_state = app.state.global_workspace.state();
        WorkspaceTabsOps::resolve_open_tabs_after_close(
            global_state.open_workspace_tabs.clone(),
            global_state.active_workspace.clone(),
            &path_str,
        )
    };
    {
        let global_state = app.state.global_workspace.state_mut();
        global_state.open_workspace_tabs = next_state.open_tabs;
        global_state.active_workspace = next_state.active_workspace.clone();
    }
    let _ = app.state.global_workspace.save();
    if !is_active {
        return;
    }
    if let Some(next_path) = next_state.active_workspace {
        let path = std::path::PathBuf::from(next_path);
        app.state.workspace.scroll_to_workspace_tab = Some(path.clone());
        super::open::WorkspaceOpenHandlersOps::handle_open_explorer(app, path);
    } else {
        app.state
            .config
            .settings
            .settings_mut()
            .workspace
            .last_workspace = None;
        let _ = app.state.config.try_save_settings();
        app.pending_action = crate::app_state::AppAction::CloseWorkspace;
    }
}

pub(super) fn handle_reorder_workspace_tab(
    app: &mut crate::shell::KatanaApp,
    from: usize,
    to: usize,
) {
    let next_state = {
        let global_state = app.state.global_workspace.state();
        WorkspaceTabsOps::resolve_open_tabs_after_reorder(
            global_state.open_workspace_tabs.clone(),
            global_state.active_workspace.clone(),
            from,
            to,
        )
    };
    {
        let global_state = app.state.global_workspace.state_mut();
        global_state.open_workspace_tabs = next_state.open_tabs;
        global_state.active_workspace = next_state.active_workspace;
    }
    let _ = app.state.global_workspace.save();
}
