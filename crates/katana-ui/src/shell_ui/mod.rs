use crate::app::*;
use crate::app_state::AppAction;
use eframe::egui;

mod shell_ui_frame;
mod shell_ui_shortcuts;
mod shell_ui_update;
mod shortcut_keys;
mod types;
pub use types::ShellUiOps;

impl ShellUiOps {
    pub fn update_native_menu_strings_from_i18n() {
        crate::native_menu::NativeMenuOps::update_native_menu_strings_from_i18n();
    }

    pub(crate) fn open_folder_dialog() -> Option<std::path::PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }

    pub(crate) fn pick_open_workspace() -> AppAction {
        AppAction::PickOpenWorkspace
    }
}

pub(crate) const WORKSPACE_SPINNER_OUTER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_INNER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_TEXT_MARGIN: f32 = 5.0;
pub(crate) const STATUS_SUCCESS_GREEN: u8 = 200;
pub(crate) const STATUS_BAR_ICON_SPACING: f32 = 4.0;

pub(crate) const SEARCH_MODAL_WIDTH: f32 = 500.0;
pub(crate) const SEARCH_MODAL_HEIGHT: f32 = 400.0;
pub(crate) const TOC_PANEL_DEFAULT_WIDTH: f32 = 200.0;
pub(crate) const TOC_PANEL_MARGIN: f32 = 8.0;
pub(crate) const TOC_HEADING_VISIBILITY_THRESHOLD: f32 = 40.0;
pub(crate) const TOC_INDENT_PER_LEVEL: f32 = 18.0;
pub(crate) const LIGHT_MODE_ICON_BG: u8 = 235;
pub(crate) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 200;

pub(crate) struct TreeRenderContext<'a, 'b> {
    pub action: &'a mut AppAction,
    pub depth: usize,
    pub active_path: Option<&'b std::path::Path>,
    pub filter_set: Option<&'b std::collections::HashSet<std::path::PathBuf>>,
    pub expanded_directories: &'a mut std::collections::HashSet<std::path::PathBuf>,
    pub disable_context_menu: bool,
    pub is_flat_view: bool,
    pub ws_root: Option<&'b std::path::Path>,
    pub tab_groups: Option<&'b [crate::state::document::TabGroup]>,
    pub show_vertical_line: bool,
}

use crate::shell::KatanaApp;

pub(crate) const SPLIT_HALF_RATIO: f32 = 0.5;
pub(crate) const SPLIT_PANEL_MAX_RATIO: f32 = 0.7;
pub(crate) const PREVIEW_CONTENT_PADDING: i8 = 12;

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme_colors = self.poll_and_prepare(ctx);

        if !self.show_main_panels(ctx, &theme_colors) {
            return;
        }

        if self.is_foreground_surface_active(ctx) {
            egui::Area::new("foreground_surface_blocker".into())
                .order(egui::Order::Background)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .interactable(true)
                .show(ctx, |ui| {
                    ui.allocate_rect(ctx.screen_rect(), egui::Sense::all());
                });
        }

        self.show_modals(ctx);

        self.state.scroll.scroll_to_line = None;
        crate::views::app_frame::AppFrameOps::intercept_url_commands(ctx, self);

        self.show_splash(ctx);
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        /* WHY: eframe::App trait requires this method in some contexts or versions. */
    }

    fn on_exit(&mut self) {
        self.save_workspace_state();
    }
}

#[cfg(test)]
include!("shell_ui_tests.rs");
