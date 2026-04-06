#![allow(clippy::useless_vec)]

use katana_platform::FilesystemService;

use crate::app::*;

use crate::{
    app_state::{AppAction, AppState},
    preview_pane::PreviewPane,
};
mod types;
pub(crate) use types::{ExplorerLoadType, ExportTask, TabPreviewCache};
pub use types::{KatanaApp, UpdateInstallEvent};

pub(crate) const SIDEBAR_COLLAPSED_TOGGLE_WIDTH: f32 = 24.0;

pub(crate) const FILE_TREE_PANEL_MIN_WIDTH: f32 = 120.0;

pub(crate) const FILE_TREE_PANEL_DEFAULT_WIDTH: f32 = 220.0;

pub(crate) const SPLIT_PREVIEW_PANEL_MIN_WIDTH: f32 = 200.0;

pub(crate) const TAB_NAV_BUTTONS_AREA_WIDTH: f32 = 80.0;

pub(crate) const TAB_INTER_ITEM_SPACING: f32 = 4.0;

pub(crate) const TAB_DROP_ANIMATION_TIME: f32 = 0.1;

pub(crate) const TAB_DROP_INDICATOR_WIDTH: f32 = 2.5;

pub(crate) const EDITOR_INITIAL_VISIBLE_ROWS: usize = 40;

pub(crate) const SCROLL_SYNC_DEAD_ZONE: f32 = 0.002;

pub(crate) const TAB_TOOLTIP_SHOW_DELAY_SECS: f32 = 0.25;

pub(crate) const NO_WORKSPACE_BOTTOM_SPACING: f32 = 8.0;

pub(crate) const RECENT_WORKSPACES_SPACING: f32 = 8.0;

pub(crate) const RECENT_WORKSPACES_ITEM_SPACING: f32 = 4.0;

pub(crate) const RECENT_WORKSPACES_CLOSE_BUTTON_WIDTH: f32 = 20.0;
pub(crate) const RECENT_WORKSPACES_HEADING_LEFT_PADDING: f32 = 10.0;
pub(crate) const RECENT_WORKSPACES_LIST_LEFT_PADDING: f32 = 16.0;
pub(crate) const HISTORY_MODAL_EMPTY_BOTTOM_SPACING: f32 = 10.0;

pub(crate) const TREE_ROW_HEIGHT: f32 = 22.0;

pub(crate) const TREE_LABEL_HOFFSET: f32 = 4.0;

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

        katana_core::update::UpdateCleanupOps::perform_background_cleanup();
        tracing::debug!("KatanaApp::new: Background cleanup done");
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
            app.pending_action = AppAction::OpenWorkspace(std::path::PathBuf::from(ws_path));
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

    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
    }

    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    #[doc(hidden)]
    pub fn set_changelog_sections_for_test(
        &mut self,
        sections: Vec<crate::changelog::ChangelogSection>,
    ) {
        self.changelog_sections = sections;
    }

    pub fn clear_changelog_rx_for_test(&mut self) {
        self.changelog_rx = None;
    }
}

impl KatanaApp {
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }
}

#[cfg(test)]
include!("shell_tests.rs");
