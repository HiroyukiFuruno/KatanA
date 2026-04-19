use crate::app_state::AppAction;
use eframe::egui;

pub use super::side_panel_types::*;

pub(super) const PREVIEW_SIDE_BAR_WIDTH: f32 = 40.0;
pub(super) const PREVIEW_SIDE_BAR_MARGIN: f32 = 4.0;
pub(super) const PREVIEW_SIDE_BAR_SPACING: f32 = 4.0;
pub(super) const LIGHT_MODE_ICON_BG: u8 = 245;
pub(super) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 230;
pub const PANEL_WIDTH: f32 = 260.0;
pub const SELECTABLE_H: f32 = 20.0;
pub(super) const PANEL_HEAD_SPACE: f32 = 8.0;
pub(super) const PANEL_ITEM_SPACE: f32 = 4.0;
pub(super) const PANEL_HOVER_MARGIN: f32 = 12.0;
pub(super) const PANEL_ANIM_SPEED: f32 = 0.15;
pub(super) const TOGGLE_BUTTON_SIZE: f32 = 32.0;
pub(super) const TOGGLE_BUTTON_ROUNDING: u8 = 4;
pub(super) const POPUP_ROUNDING: f32 = 8.0;
pub(super) const POPUP_PADDING: i8 = 0;
pub(super) const POPUP_SHADOW_ALPHA: u8 = 48;
pub(super) const POPUP_GAP: f32 = 2.0;

/// Delay (seconds) before switching to a different popup panel on hover.
///
/// WHY: Prevents accidental panel switches when the user moves the mouse
/// across toggle buttons while trying to reach the active popup content.
pub(super) const HOVER_SWITCH_DELAY: f64 = 0.25;

impl<'a> PreviewSidePanels<'a> {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.render_sidebar(ui);
        self.render_export(ui);
        self.render_story(ui);
        self.render_tools(ui);
        self.render_toc(ui);
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        let panel_resp = egui::SidePanel::right("preview_side_bar")
            .resizable(false)
            .exact_width(PREVIEW_SIDE_BAR_WIDTH)
            .show_inside(ui, |ui| {
                ui.add_space(PREVIEW_SIDE_BAR_MARGIN);
                ui.vertical_centered(|ui| {
                    let i18n = crate::i18n::I18nOps::get();

                    let toc_visible = self.app.state.config.settings.settings().layout.toc_visible;
                    if toc_visible {
                        let resp_toc = self.render_toggle_button(
                            ui,
                            crate::Icon::Toc,
                            self.app.state.layout.show_toc,
                            &i18n.action.toggle_toc,
                        );
                        self.toc_btn_rect = Some(resp_toc.rect);
                        if resp_toc.clicked() {
                            self.app.pending_action = AppAction::ToggleToc;
                            /* WHY: Set cooldown so hover does not re-open immediately
                             * when the cursor stays on the button after a click. */
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("toc_hover_cooldown"), true);
                            });
                        }
                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);
                    }

                    let resp_refresh = self.render_toggle_button(
                        ui,
                        crate::Icon::Refresh,
                        false,
                        &i18n.action.refresh_document,
                    );
                    if resp_refresh.clicked() {
                        self.app.pending_action = AppAction::RefreshDocument { is_manual: true };
                    }
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    let doc_search_tooltip = format!(
                        "{} ({})",
                        i18n.search.doc_search_title,
                        crate::os_command::OsCommandOps::get("search_workspace")
                    );
                    let resp_search = self.render_toggle_button(
                        ui,
                        crate::Icon::Search,
                        self.app.state.search.doc_search_open,
                        &doc_search_tooltip,
                    );
                    if resp_search.clicked() {
                        self.app.pending_action = AppAction::ToggleDocSearch;
                    }
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    let resp_export = self.render_toggle_button(
                        ui,
                        crate::Icon::Export,
                        self.app.state.layout.show_export_panel,
                        &i18n.menu.export,
                    );
                    self.export_btn_rect = Some(resp_export.rect);
                    if resp_export.clicked() {
                        self.app.pending_action = AppAction::ToggleExportPanel;
                    }
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    let resp_story = self.render_toggle_button(
                        ui,
                        crate::Icon::Preview,
                        self.app.state.layout.show_story_panel,
                        &i18n.preview.slideshow_settings,
                    );
                    self.story_btn_rect = Some(resp_story.rect);
                    if resp_story.clicked() {
                        self.app.pending_action = AppAction::ToggleStoryPanel;
                    }
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    let resp_tools = self.render_toggle_button(
                        ui,
                        crate::Icon::Tools,
                        self.app.state.layout.show_tools_panel,
                        &i18n.menu.view,
                    );
                    self.tools_btn_rect = Some(resp_tools.rect);
                    if resp_tools.clicked() {
                        self.app.pending_action = AppAction::ToggleToolsPanel;
                    }
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    /* 7. Meta Info */
                    let resp_info = self.render_toggle_button(
                        ui,
                        crate::Icon::Info,
                        false,
                        &i18n.meta_info.title,
                    );
                    if resp_info.clicked()
                        && let Some(doc) = self.app.state.active_document()
                    {
                        self.app.pending_action = AppAction::ShowMetaInfo(doc.path.clone());
                    }

                    /* WHY: Popup hover (Export/Story/Tools) is centralized here. */
                    self.handle_popup_hover(
                        ui,
                        resp_export.hovered(),
                        resp_story.hovered(),
                        resp_tools.hovered(),
                    );
                    /* WHY: TOC hover is managed entirely inside render_toc to avoid
                     * data_mut calls competing with resp_toc.clicked() event state. */
                });
            });
        self.sidebar_rect = Some(panel_resp.response.rect);
    }

    pub(super) fn render_toggle_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: crate::Icon,
        is_active: bool,
        tooltip: &str,
    ) -> egui::Response {
        #[rustfmt::skip]
        let icon_bg = if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG) };
        #[rustfmt::skip]
        let active_bg = if ui.visuals().dark_mode { ui.visuals().selection.bg_fill } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG) };
        let bg = if is_active { active_bg } else { icon_bg };

        let resp = ui
            .add(
                egui::Button::image(icon.ui_image(ui, crate::icon::IconSize::Medium))
                    .fill(bg)
                    .min_size(egui::vec2(TOGGLE_BUTTON_SIZE, TOGGLE_BUTTON_SIZE))
                    .rounding(egui::Rounding::same(TOGGLE_BUTTON_ROUNDING)),
            )
            .on_hover_text(tooltip);

        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tooltip)
        });

        resp
    }
}
