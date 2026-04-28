#![allow(clippy::useless_vec)]

use katana_platform::FilesystemService;

use crate::app::*;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::PreviewPane,
};
mod test_hooks;
mod transient_workspace;
mod types;
pub(crate) use types::{ExplorerLoadType, ExportTask, TabPreviewCache};
pub use types::{KatanaApp, UpdateInstallEvent};

pub mod constants;
pub(crate) use constants::*;

pub(crate) const ACTIVITY_RAIL_PADDING: f32 = 8.0;

pub(crate) const TREE_ROW_HEIGHT: f32 = 22.0;
pub(crate) const TREE_LABEL_HOFFSET: f32 = 4.0;
pub(crate) const TREE_FONT_SIZE: f32 = 13.0;
pub(crate) const TREE_INDENT_STEP: f32 = 12.0;
pub(crate) const TREE_ICON_ARROW_GAP: f32 = 4.0;
pub(crate) const TREE_ICON_LABEL_GAP: f32 = 6.0;
pub(crate) const TREE_HOVER_ROUNDING: f32 = 2.0;
pub(crate) const TREE_HOVER_GAMMA: f32 = 0.4;
pub(crate) const TREE_DRAG_GHOST_GAMMA: f32 = 0.7;
pub(crate) const TREE_ACCORDION_LINE_OFFSET: f32 = 6.0;
pub(crate) const TREE_ACCORDION_LINE_WIDTH: f32 = 1.0;
pub(crate) const TREE_ACCORDION_LINE_GAMMA: f32 = 0.2;
pub(crate) const TREE_ACCORDION_LINE_DASH_LENGTH: f32 = 1.0;
pub(crate) const TREE_ACCORDION_LINE_GAP_LENGTH: f32 = 3.0;

pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;
pub(crate) const ACTIVE_FILE_HIGHLIGHT_ROUNDING: f32 = 3.0;

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        tracing::debug!("KatanaApp::new: Start");
        let mut app = Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_previews: Vec::new(),
            download_rx: None,
            active_download: None,
            renderer_asset_rx: None,
            explorer_rx: None,
            update_rx: None,
            changelog_rx: None,
            update_install_rx: None,
            export_tasks: Vec::new(),
            pending_document_loads: std::collections::VecDeque::new(),
            pending_workspace_file_open: None,
            linter_doc_rx: None,
            linter_docs_cache: std::collections::HashMap::new(),
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
            file_dialog: egui_file_dialog::FileDialog::new(),
            pending_dialog_action: None,
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

        app.clear_transient_workspace_restore_state();
        katana_core::update::UpdateCleanupOps::perform_background_cleanup();
        tracing::debug!("KatanaApp::new: Background cleanup done");
        if !cfg!(test) {
            app.start_renderer_asset_bootstrap();
        }
        app.start_update_check(false);
        tracing::debug!("KatanaApp::new: End");

        let last_ws = app
            .state
            .config
            .settings
            .settings()
            .workspace
            .last_workspace
            .clone();
        if let Some(ws_path) = last_ws {
            if std::path::Path::new(&ws_path).exists() {
                app.pending_action = AppAction::OpenWorkspace(std::path::PathBuf::from(ws_path));
            } else {
                tracing::warn!(
                    "Saved workspace path no longer exists, skipping restore: {}",
                    ws_path
                );
            }
        }

        app
    }

    pub fn skip_splash(&mut self) {
        self.needs_splash = false;
        self.splash_start = None;
    }

    #[doc(hidden)]
    pub fn open_update_dialog_for_test(&mut self) {
        self.show_update_dialog = true;
    }

    #[doc(hidden)]
    pub fn disable_changelog_display_for_test(&mut self) {
        self.needs_changelog_display = false;
    }
}

impl KatanaApp {
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    pub fn is_foreground_surface_active(&self, ctx: &egui::Context) -> bool {
        self.state.layout.show_settings
            || self.state.layout.show_workspace_panel
            || self.state.layout.show_history_panel
            || self.state.layout.show_slideshow
            || self.state.layout.show_search_modal
            || self.state.layout.create_fs_node_modal.is_some()
            || self.state.layout.rename_modal.is_some()
            || self.state.layout.delete_modal.is_some()
            || self.state.command_palette.is_open
            || self.show_about
            || self.show_update_dialog
            || self.needs_splash
            || self.splash_start.is_some()
            || self.show_meta_info_for.is_some()
            || self.needs_changelog_display
            || ctx.memory(|mem| mem.any_popup_open())
    }
}

#[cfg(test)]
include!("shell_tests.rs");
