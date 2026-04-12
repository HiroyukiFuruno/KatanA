use crate::app::*;
use crate::app_state::*;
use crate::preview_pane::PreviewPane;
use crate::shell::*;
use katana_platform::FilesystemService;

pub(super) fn build_katana_app(state: AppState) -> KatanaApp {
    let mut app = KatanaApp {
        state,
        fs: FilesystemService::new(),
        pending_action: AppAction::None,
        tab_previews: Vec::new(),
        download_rx: None,
        explorer_rx: None,
        update_rx: None,
        changelog_rx: None,
        update_install_rx: None,
        export_tasks: Vec::new(),
        pending_document_loads: std::collections::VecDeque::new(),
        show_about: false,
        show_update_dialog: false,
        update_markdown_cache: egui_commonmark::CommonMarkCache::default(),
        update_notified: false,
        about_icon: None,
        cached_theme: None,
        cached_font_size: None,
        cached_font_family: None,
        settings_preview: PreviewPane::default(),
        needs_splash: !cfg!(test),
        splash_start: None,
        show_meta_info_for: None,
        pending_relaunch: None,
        changelog_sections: Vec::new(),
        needs_changelog_display: false,
        old_app_version: None,
        editor_cursor_range: None,
        pending_editor_cursor: None,
    };
    let current_version = env!("CARGO_PKG_VERSION");
    let mut show_changelog = false;
    {
        let settings_mut = app.state.config.settings.settings_mut();
        if let Some(prev) = &settings_mut.updates.previous_app_version {
            app.old_app_version = Some(prev.clone());
            if prev != current_version {
                show_changelog = true;
            }
        } else {
            show_changelog = true;
        }
        if show_changelog {
            settings_mut.updates.previous_app_version = Some(current_version.to_string());
        }
    }
    if show_changelog {
        if !app.state.config.try_save_settings() {
            tracing::warn!("Failed to save previous_app_version");
        }
        app.needs_changelog_display = true;
    }
    app.start_update_check(false);
    app
}
