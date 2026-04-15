use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

pub(super) const PREVIEW_SIDE_BAR_WIDTH: f32 = 40.0;
pub(super) const PREVIEW_SIDE_BAR_MARGIN: f32 = 4.0;
pub(super) const PREVIEW_SIDE_BAR_SPACING: f32 = 4.0;
pub(super) const LIGHT_MODE_ICON_BG: u8 = 245;
pub(super) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 230;

/* WHY: Shared UI constants for the slide-out panels. */
pub const PANEL_WIDTH: f32 = 260.0;
pub const SELECTABLE_H: f32 = 20.0;
pub(super) const PANEL_HEAD_SPACE: f32 = 8.0;
pub(super) const PANEL_ITEM_SPACE: f32 = 4.0;
pub(super) const PANEL_HOVER_MARGIN: f32 = 12.0;
pub(super) const PANEL_ANIM_SPEED: f32 = 0.15;
pub(super) const TOGGLE_BUTTON_SIZE: f32 = 32.0;
pub(super) const TOGGLE_BUTTON_ROUNDING: u8 = 4;

pub struct PreviewSidePanels<'a> {
    pub app: &'a mut KatanaApp,
    pub(super) export_btn_rect: Option<egui::Rect>,
    pub(super) story_btn_rect: Option<egui::Rect>,
    pub(super) tools_btn_rect: Option<egui::Rect>,
    pub(super) toc_btn_rect: Option<egui::Rect>,
}

impl<'a> PreviewSidePanels<'a> {
    pub fn new(app: &'a mut KatanaApp) -> Self {
        Self {
            app,
            export_btn_rect: None,
            story_btn_rect: None,
            tools_btn_rect: None,
            toc_btn_rect: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.render_sidebar(ui);
        self.render_export(ui);
        self.render_story(ui);
        self.render_tools(ui);
        self.render_toc(ui);
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::right("preview_side_bar")
            .resizable(false)
            .exact_width(PREVIEW_SIDE_BAR_WIDTH)
            .show_inside(ui, |ui| {
                ui.add_space(PREVIEW_SIDE_BAR_MARGIN);
                ui.vertical_centered(|ui| {
                    let i18n = crate::i18n::I18nOps::get();
                    let view_mode = self.app.state.active_view_mode();
                    let is_code_only = view_mode == crate::app_state::ViewMode::CodeOnly;

                    /* 1. 目次 */
                    if !is_code_only {
                        let resp_toc = self.render_toggle_button(
                            ui,
                            crate::Icon::Toc,
                            self.app.state.layout.show_toc,
                            &i18n.toc.title,
                        );
                        self.toc_btn_rect = Some(resp_toc.rect);
                        if resp_toc.clicked() {
                            self.app.pending_action = AppAction::ToggleToc;
                        }

                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);
                    }

                    /* 2. ドキュメント更新 */
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

                    /* 3. 検索 */
                    let search_open = self.app.state.search.doc_search_open;
                    let doc_search_tooltip = format!(
                        "{} ({})",
                        i18n.search.doc_search_title,
                        crate::os_command::OsCommandOps::get("search_workspace")
                    );
                    let resp_search = self.render_toggle_button(
                        ui,
                        crate::Icon::Search,
                        search_open,
                        &doc_search_tooltip,
                    );
                    if resp_search.clicked() {
                        if search_open {
                            self.app.state.search.doc_search_open = false;
                        } else {
                            self.app.pending_action = AppAction::OpenDocSearch;
                        }
                    }

                    if !is_code_only {
                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                        /* 4. エクスポート */
                        let resp_export = self.render_toggle_button(
                            ui,
                            crate::Icon::Export,
                            self.app.state.layout.show_export_panel,
                            &i18n.menu.export,
                        );
                        self.export_btn_rect = Some(resp_export.rect);
                        if resp_export.hovered() {
                            self.app.state.layout.show_export_panel = true;
                            self.app.state.layout.show_story_panel = false;
                            self.app.state.layout.show_tools_panel = false;
                            self.app.state.layout.show_toc = false;
                        }
                        if resp_export.clicked() {
                            self.app.pending_action = AppAction::ToggleExportPanel;
                        }

                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                        /* 5. スライドショー */
                        let resp_story = self.render_toggle_button(
                            ui,
                            crate::Icon::Preview,
                            self.app.state.layout.show_story_panel,
                            &i18n.preview.slideshow_settings,
                        );
                        self.story_btn_rect = Some(resp_story.rect);
                        if resp_story.hovered() {
                            self.app.state.layout.show_story_panel = true;
                            self.app.state.layout.show_export_panel = false;
                            self.app.state.layout.show_tools_panel = false;
                            self.app.state.layout.show_toc = false;
                        }
                        if resp_story.clicked() {
                            self.app.pending_action = AppAction::ToggleStoryPanel;
                        }
                    }

                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    /* 6. 表示 */
                    let resp_tools = self.render_toggle_button(
                        ui,
                        crate::Icon::Tools,
                        self.app.state.layout.show_tools_panel,
                        &i18n.menu.view,
                    );
                    self.tools_btn_rect = Some(resp_tools.rect);
                    if resp_tools.hovered() {
                        self.app.state.layout.show_tools_panel = true;
                        self.app.state.layout.show_export_panel = false;
                        self.app.state.layout.show_story_panel = false;
                        self.app.state.layout.show_toc = false;
                    }
                    if resp_tools.clicked() {
                        self.app.pending_action = AppAction::ToggleToolsPanel;
                    }

                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    /* 7. メタ情報 */
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

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |_ui| {});
                });
            });
    }

    pub(super) fn render_toggle_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: crate::Icon,
        is_active: bool,
        tooltip: &str,
    ) -> egui::Response {
        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG)
        };

        let active_bg = if ui.visuals().dark_mode {
            ui.visuals().selection.bg_fill
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
        };

        let bg = if is_active { active_bg } else { icon_bg };

        ui.add(
            egui::Button::image(icon.ui_image(ui, crate::icon::IconSize::Medium))
                .fill(bg)
                .min_size(egui::vec2(TOGGLE_BUTTON_SIZE, TOGGLE_BUTTON_SIZE))
                .rounding(egui::Rounding::same(TOGGLE_BUTTON_ROUNDING)),
        )
        .on_hover_text(tooltip)
    }
}
