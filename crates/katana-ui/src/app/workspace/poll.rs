use super::*;

pub(super) fn handle_refresh_explorer(app: &mut KatanaApp) {
    let Some(workspace) = &app.state.workspace.data else {
        return;
    };
    let root = workspace.root.clone();
    app.state.workspace.is_loading = true;
    let (tx, rx) = std::sync::mpsc::channel();
    app.explorer_rx = Some(rx);
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
            &root,
            &settings.ignored_directories,
            settings.max_depth,
            &settings.visible_extensions,
            &settings.extensionless_excludes,
            new_token,
            &in_memory_dirs,
        );
        let _ = tx.send((ExplorerLoadType::Refresh, root, result));
    });
}

pub(super) fn poll_explorer_load(app: &mut KatanaApp, ctx: &egui::Context) {
    const WORKSPACE_LOAD_POLL_INTERVAL_MS: u64 = 50;
    let Some(rx) = &app.explorer_rx else { return };
    let recv_result = rx.try_recv();
    let done = match recv_result {
        Ok((ExplorerLoadType::Open, path, Ok(ws))) => {
            app.state.workspace.is_loading = false;
            open::finish_open_explorer(app, path, ws);
            true
        }
        Ok((ExplorerLoadType::Refresh, _path, Ok(ws))) => {
            app.state.workspace.is_loading = false;
            app.state.workspace.data = Some(ws);
            app.state.search.filter_cache = None;
            true
        }
        Ok((_load_type, _path, Err(e))) => {
            app.state.workspace.is_loading = false;
            let error = e.to_string();
            app.state.layout.status_message = Some((
                crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().status.cannot_open_workspace,
                    &[("error", error.as_str())],
                ),
                crate::app_state::StatusType::Error,
            ));
            true
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            ctx.request_repaint_after(std::time::Duration::from_millis(
                WORKSPACE_LOAD_POLL_INTERVAL_MS,
            ));
            false
        }
        Err(_) => {
            app.state.workspace.is_loading = false;
            true
        }
    };
    if done {
        app.explorer_rx = None;
    }
    if app.needs_changelog_display
        && !app.state.workspace.is_loading
        && app.explorer_rx.is_none()
        && matches!(app.pending_action, AppAction::None)
    {
        app.needs_changelog_display = false;
        app.pending_action = AppAction::ShowReleaseNotes;
    }
}
