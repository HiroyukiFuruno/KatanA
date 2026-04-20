/* WHY: Isolated workspace loading handlers to coordinate asynchronous filesystem operations and UI feedback. */

use super::session::WorkspaceOpenSessionOps;
use crate::app::workspace::ExplorerLoadType;
use crate::app::workspace::manage;
use crate::shell::KatanaApp;

pub struct WorkspaceOpenHandlersOps;

impl WorkspaceOpenHandlersOps {
    pub fn handle_open_explorer(app: &mut KatanaApp, path: std::path::PathBuf) {
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
        /* WHY: Token logic removed here to avoid redundancy and E0283 */
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

    pub fn finish_open_explorer(
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
        let (to_open, active_idx) = WorkspaceOpenSessionOps::restore_session_tabs(app, &path_str);
        WorkspaceOpenSessionOps::apply_session_tabs(app, to_open, active_idx, path_str);
    }
}
