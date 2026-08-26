pub use super::side_panel_types::*;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) const PREVIEW_SIDE_BAR_WIDTH: f32 = 32.0;
pub(super) const PREVIEW_SIDE_BAR_MARGIN: f32 = 2.0;
pub(super) const PREVIEW_SIDE_BAR_SPACING: f32 = 4.0;
pub(super) const LIGHT_MODE_ICON_BG: u8 = 245;
pub const PANEL_WIDTH: f32 = 260.0;
pub const SELECTABLE_H: f32 = 20.0;
pub(super) const PANEL_HEAD_SPACE: f32 = 8.0;
pub(super) const PANEL_ITEM_SPACE: f32 = 4.0;
pub(super) const PANEL_HOVER_MARGIN: f32 = 12.0;
pub(super) const PANEL_ANIM_SPEED: f32 = 0.15;
pub(super) const POPUP_ROUNDING: f32 = 8.0;
pub(super) const POPUP_PADDING: i8 = 0;
pub(super) const POPUP_SHADOW_ALPHA: u8 = 48;
pub(super) const POPUP_GAP: f32 = 2.0;

/* WHY: Prevents accidental panel switches while the pointer crosses buttons. */
pub(super) const HOVER_SWITCH_DELAY: f64 = 0.25;

impl<'a> PreviewSidePanels<'a> {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !super::TocAvailability::for_path(
            self.app
                .state
                .active_document()
                .map(|document| document.path.as_path()),
        ) {
            self.app.state.layout.show_toc = false;
        }
        self.render_sidebar(ui);
        self.render_export(ui);
        self.render_story(ui);
        self.render_tools(ui);
        self.render_toc(ui);
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        let panel_resp = egui::Panel::right("preview_side_bar")
            .resizable(false)
            .exact_size(PREVIEW_SIDE_BAR_WIDTH)
            .show_inside(ui, |ui| {
                ui.add_space(PREVIEW_SIDE_BAR_MARGIN);
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing.y = PREVIEW_SIDE_BAR_SPACING;
                    let i18n = crate::i18n::I18nOps::get();

                    let toc_visible = self.app.state.config.settings.settings().layout.toc_visible;
                    if toc_visible {
                        let toc_available = super::TocAvailability::for_path(
                            self.app
                                .state
                                .active_document()
                                .map(|document| document.path.as_path()),
                        );
                        let resp_toc = ui
                            .add_enabled_ui(toc_available, |ui| {
                                self.render_toggle_button(
                                    ui,
                                    crate::Icon::Toc,
                                    self.app.state.layout.show_toc,
                                    &i18n.action.toggle_toc,
                                    None,
                                )
                            })
                            .inner;
                        self.toc_btn_rect = Some(resp_toc.rect);
                        if resp_toc.clicked() {
                            self.app.pending_action = AppAction::ToggleToc;
                            /* WHY: Set cooldown so hover does not re-open immediately
                             * when the cursor stays on the button after a click. */
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("toc_hover_cooldown"), true);
                            });
                        }
                    }

                    let resp_refresh = self.render_toggle_button(
                        ui,
                        crate::Icon::Refresh,
                        false,
                        &i18n.action.refresh_document,
                        None,
                    );
                    if resp_refresh.clicked() {
                        self.app.pending_action = AppAction::RefreshDocument { is_manual: true };
                    }

                    let doc_search_shortcut = crate::os_command::OsCommandOps::get("search_tab");
                    let resp_search = self.render_toggle_button(
                        ui,
                        crate::Icon::Search,
                        self.app.state.search.doc_search_open,
                        &i18n.search.doc_search_title,
                        Some(&doc_search_shortcut),
                    );
                    if resp_search.clicked() {
                        self.app.pending_action = AppAction::ToggleDocSearch;
                    }

                    let resp_export = self.render_toggle_button(
                        ui,
                        crate::Icon::Export,
                        self.app.state.layout.show_export_panel,
                        &i18n.menu.export,
                        None,
                    );
                    self.export_btn_rect = Some(resp_export.rect);
                    if resp_export.clicked() {
                        self.app.pending_action = AppAction::ToggleExportPanel;
                    }

                    let resp_story = self.render_toggle_button(
                        ui,
                        crate::Icon::Preview,
                        self.app.state.layout.show_story_panel,
                        &i18n.preview.slideshow_settings,
                        None,
                    );
                    self.story_btn_rect = Some(resp_story.rect);
                    if resp_story.clicked() {
                        self.app.pending_action = AppAction::ToggleStoryPanel;
                    }

                    let resp_tools = self.render_toggle_button(
                        ui,
                        crate::Icon::Tools,
                        self.app.state.layout.show_tools_panel,
                        &i18n.menu.view,
                        None,
                    );
                    self.tools_btn_rect = Some(resp_tools.rect);
                    if resp_tools.clicked() {
                        self.app.pending_action = AppAction::ToggleToolsPanel;
                    }

                    /* 7. Meta Info */
                    let resp_info = self.render_toggle_button(
                        ui,
                        crate::Icon::Info,
                        false,
                        &i18n.meta_info.title,
                        None,
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
        ui.painter().line_segment(
            [
                panel_resp.response.rect.left_top(),
                panel_resp.response.rect.left_bottom(),
            ],
            ui.visuals().window_stroke(),
        );
    }
}
