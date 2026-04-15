use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_ui::{LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG};
use eframe::egui;

const PREVIEW_SIDE_BAR_WIDTH: f32 = 40.0;
const PREVIEW_SIDE_BAR_MARGIN: f32 = 4.0;
const PREVIEW_SIDE_BAR_SPACING: f32 = 4.0;
const TOGGLE_BUTTON_SIZE: f32 = 32.0;
const TOGGLE_BUTTON_ROUNDING: u8 = 4;

pub struct PreviewSidePanels<'a> {
    pub app: &'a mut KatanaApp,
    export_btn_rect: Option<egui::Rect>,
    story_btn_rect: Option<egui::Rect>,
    tools_btn_rect: Option<egui::Rect>,
}

impl<'a> PreviewSidePanels<'a> {
    pub fn new(app: &'a mut KatanaApp) -> Self {
        Self {
            app,
            export_btn_rect: None,
            story_btn_rect: None,
            tools_btn_rect: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let doc_exists = self.app.state.active_document().is_some();
        if !doc_exists {
            return;
        }

        self.render_side_bar(ui);
        self.render_toc(ui);
        self.render_tools(ui);

        let is_code_only =
            self.app.state.active_view_mode() == crate::app_state::ViewMode::CodeOnly;
        if !is_code_only {
            self.render_export(ui);
            self.render_story(ui);
        }
    }

    fn render_side_bar(&mut self, ui: &mut egui::Ui) {
        let panel = egui::SidePanel::right("preview_side_bar");

        panel
            .exact_width(PREVIEW_SIDE_BAR_WIDTH)
            .resizable(false)
            .frame(
                egui::Frame::side_top_panel(&ui.ctx().global_style())
                    .inner_margin(PREVIEW_SIDE_BAR_MARGIN),
            )
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    // 1. TOC (Always shown)
                    let resp_toc = self.render_toggle_button(
                        ui,
                        crate::Icon::Toc,
                        self.app.state.layout.show_toc,
                        &crate::i18n::I18nOps::get().toc.title,
                    );
                    if resp_toc.clicked() {
                        self.app.pending_action = AppAction::ToggleToc;
                    }

                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    // 2. Refresh (Always shown)
                    let resp_refresh = self.render_toggle_button(
                        ui,
                        crate::Icon::Refresh,
                        false,
                        &crate::i18n::I18nOps::get().action.refresh_document,
                    );
                    if resp_refresh.clicked() {
                        self.app.pending_action = AppAction::RefreshDocument { is_manual: true };
                    }

                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    // 3. Search (Always shown)
                    let search_open = self.app.state.search.doc_search_open;
                    let doc_search_tooltip = format!(
                        "{} ({}F)",
                        crate::i18n::I18nOps::get().search.doc_search_title,
                        katana_platform::PlatformContractOps::PRIMARY_MODIFIER_NAME
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

                    let view_mode = self.app.state.active_view_mode();
                    let is_code_only = view_mode == crate::app_state::ViewMode::CodeOnly;

                    // 4. Export & 5. Story (Shown except in CodeOnly)
                    if !is_code_only {
                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                        let resp_export = self.render_toggle_button(
                            ui,
                            crate::Icon::Export,
                            self.app.state.layout.show_export_panel,
                            &crate::i18n::I18nOps::get().menu.export,
                        );
                        self.export_btn_rect = Some(resp_export.rect);
                        if resp_export.hovered() {
                            self.app.state.layout.show_export_panel = true;
                            self.app.state.layout.show_story_panel = false;
                            self.app.state.layout.show_tools_panel = false;
                        }
                        if resp_export.clicked() {
                            self.app.pending_action = AppAction::ToggleExportPanel;
                        }

                        ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                        let resp_story = self.render_toggle_button(
                            ui,
                            crate::Icon::Preview, // Used for Story View
                            self.app.state.layout.show_story_panel,
                            &crate::i18n::I18nOps::get().preview.slideshow_settings,
                        );
                        self.story_btn_rect = Some(resp_story.rect);
                        if resp_story.hovered() {
                            self.app.state.layout.show_story_panel = true;
                            self.app.state.layout.show_export_panel = false;
                            self.app.state.layout.show_tools_panel = false;
                        }
                        if resp_story.clicked() {
                            self.app.pending_action = AppAction::ToggleStoryPanel;
                        }
                    }

                    // 6. Tools Panel
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);

                    let resp_tools = self.render_toggle_button(
                        ui,
                        crate::Icon::Tools,
                        self.app.state.layout.show_tools_panel,
                        &crate::i18n::I18nOps::get().menu.view,
                    );
                    self.tools_btn_rect = Some(resp_tools.rect);
                    if resp_tools.hovered() {
                        self.app.state.layout.show_tools_panel = true;
                        self.app.state.layout.show_export_panel = false;
                        self.app.state.layout.show_story_panel = false;
                    }

                    if resp_tools.clicked() {
                        self.app.pending_action = AppAction::ToggleToolsPanel;
                    }

                    // 7. Meta Data / Info (Always shown)
                    ui.add_space(PREVIEW_SIDE_BAR_SPACING);
                    let resp_info = self.render_toggle_button(
                        ui,
                        crate::Icon::Info,
                        false, // Never "active"
                        &crate::i18n::I18nOps::get().meta_info.title,
                    );
                    if resp_info.clicked() {
                        if let Some(doc) = self.app.state.active_document() {
                            self.app.pending_action = AppAction::ShowMetaInfo(doc.path.clone());
                        }
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |_ui| {});
                });
            });
    }

    fn render_toggle_button(
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

    fn render_toc(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_toc {
            return;
        }

        let doc = match self.app.state.active_document() {
            Some(d) => d,
            None => return,
        };

        if let Some(preview) = self
            .app
            .tab_previews
            .iter_mut()
            .find(|p| p.path == doc.path)
        {
            let (clicked_line, active_index) =
                crate::views::panels::toc::TocPanel::new(&mut preview.pane, &mut self.app.state)
                    .show(ui);

            if let Some(clicked) = clicked_line {
                self.app.state.scroll.scroll_to_line = Some(clicked);
                self.app.state.scroll.last_scroll_to_line = None;
            }
            self.app.state.active_toc_index = active_index;
        }
    }

    fn render_export(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_export_panel {
            return;
        }

        let mut keep_open = false;
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(btn_rect) = self.export_btn_rect {
                if btn_rect.expand(12.0).contains(pos) {
                    keep_open = true;
                }
            }
        }

        let panel_resp = egui::SidePanel::right("preview_export_panel")
            .resizable(true)
            .default_width(260.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.heading(crate::i18n::I18nOps::get().menu.export.clone());
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let i18n = crate::i18n::I18nOps::get();
                    let formats = [
                        (
                            crate::Icon::Markdown,
                            i18n.menu.export_html.clone(),
                            crate::app_state::ExportFormat::Html,
                        ),
                        (
                            crate::Icon::Document,
                            i18n.menu.export_pdf.clone(),
                            crate::app_state::ExportFormat::Pdf,
                        ),
                        (
                            crate::Icon::Markdown,
                            i18n.menu.export_png.clone(),
                            crate::app_state::ExportFormat::Png,
                        ),
                        (
                            crate::Icon::Markdown,
                            i18n.menu.export_jpg.clone(),
                            crate::app_state::ExportFormat::Jpg,
                        ),
                    ];

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        for (icon, label, fmt) in formats {
                            let resp = ui.add(
                                egui::Button::image_and_text(
                                    icon.ui_image(ui, crate::icon::IconSize::Medium),
                                    label,
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(ui.available_width(), 32.0)),
                            );
                            if resp.clicked() {
                                self.app.pending_action = AppAction::ExportDocument(fmt);
                            }
                        }
                    });
                });
            });

        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if panel_resp.response.rect.expand(12.0).contains(pos) {
                keep_open = true;
            }
        }

        if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
            self.app.state.layout.show_export_panel = false;
        }
    }

    fn render_story(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_story_panel {
            return;
        }

        let mut keep_open = false;
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(btn_rect) = self.story_btn_rect {
                if btn_rect.expand(12.0).contains(pos) {
                    keep_open = true;
                }
            }
        }

        let panel_resp = egui::SidePanel::right("preview_story_panel")
            .resizable(true)
            .default_width(260.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;

                        let mut hover = self
                            .app
                            .state
                            .config
                            .settings
                            .settings()
                            .behavior
                            .slideshow_hover_highlight;
                        if ui
                            .add(
                                crate::widgets::LabeledToggle::new(
                                    &crate::i18n::I18nOps::get().preview.highlight_hover,
                                    &mut hover,
                                )
                                .position(crate::widgets::TogglePosition::Right)
                                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                            )
                            .changed()
                        {
                            self.app
                                .state
                                .config
                                .settings
                                .settings_mut()
                                .behavior
                                .slideshow_hover_highlight = hover;
                            self.app.state.layout.slideshow_hover_highlight = hover;
                            let _ = self.app.state.config.try_save_settings();
                        }

                        let mut controls = self
                            .app
                            .state
                            .config
                            .settings
                            .settings()
                            .behavior
                            .slideshow_show_diagram_controls;
                        if ui
                            .add(
                                crate::widgets::LabeledToggle::new(
                                    &crate::i18n::I18nOps::get().preview.show_diagram_controls,
                                    &mut controls,
                                )
                                .position(crate::widgets::TogglePosition::Right)
                                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                            )
                            .changed()
                        {
                            self.app
                                .state
                                .config
                                .settings
                                .settings_mut()
                                .behavior
                                .slideshow_show_diagram_controls = controls;
                            self.app.state.layout.slideshow_show_diagram_controls = controls;
                            let _ = self.app.state.config.try_save_settings();
                        }

                        ui.add_space(16.0);

                        let start_slideshow_label =
                            crate::i18n::I18nOps::get().preview.toggle_slideshow.clone();
                        if ui
                            .add(
                                egui::Button::image_and_text(
                                    crate::Icon::Preview.ui_image(ui, crate::icon::IconSize::Large),
                                    start_slideshow_label,
                                )
                                .min_size(egui::vec2(ui.available_width(), 44.0))
                                .rounding(egui::Rounding::same(8)),
                            )
                            .clicked()
                        {
                            self.app.pending_action = AppAction::ToggleSlideshow;
                        }
                    });
                });
            });

        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if panel_resp.response.rect.expand(12.0).contains(pos) {
                keep_open = true;
            }
        }

        if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
            self.app.state.layout.show_story_panel = false;
        }
    }

    fn render_tools(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_tools_panel {
            return;
        }

        let mut keep_open = false;
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(btn_rect) = self.tools_btn_rect {
                if btn_rect.expand(12.0).contains(pos) {
                    keep_open = true;
                }
            }
        }

        let panel_resp = egui::SidePanel::right("preview_tools_panel")
            .resizable(true)
            .default_width(260.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.heading(&crate::i18n::I18nOps::get().menu.view);
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;

                        let i18n = crate::i18n::I18nOps::get();
                        let view_mode = self.app.state.active_view_mode();
                        let is_split = view_mode == crate::app_state::ViewMode::Split;
                        let mut split_toggled = is_split;

                        // Split Toggle
                        if ui
                            .add(
                                crate::widgets::LabeledToggle::new(
                                    &i18n.view_mode.split,
                                    &mut split_toggled,
                                )
                                .position(crate::widgets::TogglePosition::Right)
                                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                            )
                            .changed()
                        {
                            if split_toggled {
                                self.app.pending_action = AppAction::SetViewMode(crate::app_state::ViewMode::Split);
                            } else {
                                self.app.pending_action = AppAction::SetViewMode(crate::app_state::ViewMode::PreviewOnly);
                            }
                        }

                        ui.add_space(8.0);

                        if is_split {
                            let icon_bg = if ui.visuals().dark_mode {
                                crate::theme_bridge::TRANSPARENT
                            } else {
                                crate::theme_bridge::ThemeBridgeOps::from_gray(
                                    crate::shell_ui::LIGHT_MODE_ICON_BG,
                                )
                            };
                            let mut action = None;
                            crate::views::top_bar::view_mode_split::SplitControls {
                                split_direction: self.app.state.active_split_direction(),
                                pane_order: self.app.state.active_pane_order(),
                                scroll_sync_enabled: self
                                    .app
                                    .state
                                    .config
                                    .settings
                                    .settings()
                                    .behavior
                                    .scroll_sync_enabled,
                                scroll_sync_override: self.app.state.scroll.sync_override,
                                button_size: egui::vec2(TOGGLE_BUTTON_SIZE, TOGGLE_BUTTON_SIZE),
                                icon_bg,
                                ui,
                            }
                            .show(&mut action);
                            
                            if let Some(a) = action {
                                self.app.pending_action = a;
                            }
                        } else {
                            // Tangochou (Flashcard) UI for Code | Preview
                            let is_code = view_mode == crate::app_state::ViewMode::CodeOnly;

                            let card_h = 42.0;
                            let padding = 12.0;
                            let total_w = ui.available_width();
                            let card_w = total_w - padding * 2.0;

                            let base_pos = ui.cursor().min + egui::vec2(padding, 4.0);

                            // In a real Tangochou, the hole is small and near the left edge
                            let ring_center = base_pos + egui::vec2(16.0, card_h * 0.5); 

                            // The angle the back card drops by: ~12 degrees
                            let back_angle = 12.0f32.to_radians();
                            let front_angle = 0.0f32;

                            // The back card drops down by card_w * sin(back_angle)
                            let drop_y = (card_w - 16.0) * back_angle.sin();
                            let needed_h = card_h + drop_y + 12.0;

                            let (_alloc_rect, response) = ui.allocate_exact_size(
                                egui::vec2(total_w, needed_h),
                                egui::Sense::click()
                            );

                            // Interactive hover effect: fan out slightly more
                            let (actual_back_angle, actual_front_angle) = if response.hovered() {
                                (back_angle + 2.0f32.to_radians(), front_angle - 1.5f32.to_radians())
                            } else {
                                (back_angle, front_angle)
                            };

                            let is_dark = ui.visuals().dark_mode;
                            let back_bg = if is_dark { egui::Color32::from_gray(35) } else { egui::Color32::from_gray(225) };
                            let front_bg = if is_dark { egui::Color32::from_gray(65) } else { egui::Color32::from_gray(255) };
                            let stroke_col = ui.visuals().widgets.noninteractive.bg_stroke.color;
                            let selection_col = ui.visuals().selection.bg_fill;

                            // Helper to draw a single flashcard
                            let draw_card = |angle: f32, bg: egui::Color32, text: &str, is_front: bool| {
                                let rect = egui::Rect::from_min_size(base_pos, egui::vec2(card_w, card_h));
                                
                                let rotate_pos = |p: egui::Pos2| -> egui::Pos2 {
                                    let dx = p.x - ring_center.x;
                                    let dy = p.y - ring_center.y;
                                    let cos_a = angle.cos();
                                    let sin_a = angle.sin();
                                    egui::Pos2::new(
                                        ring_center.x + dx * cos_a - dy * sin_a,
                                        ring_center.y + dx * sin_a + dy * cos_a,
                                    )
                                };

                                let corners = [
                                    rotate_pos(rect.left_top()),
                                    rotate_pos(rect.right_top()),
                                    rotate_pos(rect.right_bottom()),
                                    rotate_pos(rect.left_bottom()),
                                ];

                                if is_front {
                                    let shadow_corners = corners.map(|p| p + egui::vec2(2.0, 4.0));
                                    ui.painter().add(egui::Shape::convex_polygon(
                                        shadow_corners.into(),
                                        egui::Color32::from_black_alpha(45),
                                        egui::Stroke::NONE,
                                    ));
                                }

                                let border = if is_front { egui::Stroke::new(1.5, selection_col) } else { egui::Stroke::new(1.0, stroke_col) };
                                
                                ui.painter().add(egui::Shape::convex_polygon(
                                    corners.into(),
                                    bg,
                                    border,
                                ));

                                // Card hole cut-out (showing background)
                                let hole_pos = rotate_pos(ring_center);
                                ui.painter().circle_filled(hole_pos, 5.0, ui.visuals().window_fill());
                                ui.painter().circle_stroke(hole_pos, 5.0, egui::Stroke::new(1.0, stroke_col));

                                // Text rendering
                                let text_color = if is_front { ui.visuals().text_color() } else { ui.visuals().text_color().linear_multiply(0.6) };
                                let galley = ui.painter().layout_no_wrap(text.to_owned(), egui::FontId::proportional(15.0), text_color);
                                
                                // Position text centered in the card (slightly offset to account for hole)
                                let text_center_unrotated = rect.center() + egui::vec2(8.0, 0.0);
                                let text_top_left_unrotated = text_center_unrotated - galley.size() / 2.0;

                                let mut shape = egui::epaint::TextShape::new(rotate_pos(text_top_left_unrotated), galley, text_color);
                                shape.angle = angle;
                                ui.painter().add(egui::Shape::Text(shape));
                            };

                            let (front_text, back_text) = if is_code {
                                (&i18n.view_mode.code, &i18n.view_mode.preview)
                            } else {
                                (&i18n.view_mode.preview, &i18n.view_mode.code)
                            };

                            draw_card(actual_back_angle, back_bg, back_text, false);
                            draw_card(actual_front_angle, front_bg, front_text, true);

                            // The Binder Ring
                            let ring_color = if is_dark { egui::Color32::from_gray(120) } else { egui::Color32::from_gray(180) };
                            ui.painter().circle_stroke(ring_center, 8.0, egui::Stroke::new(3.0, ring_color));
                            ui.painter().circle_stroke(ring_center, 9.5, egui::Stroke::new(1.0, egui::Color32::from_black_alpha(60)));
                            ui.painter().circle_stroke(ring_center, 6.5, egui::Stroke::new(1.0, egui::Color32::from_black_alpha(60)));
                            // Highlight on ring
                            ui.painter().line_segment(
                                [ring_center - egui::vec2(4.0, 6.0), ring_center - egui::vec2(1.0, 8.0)],
                                egui::Stroke::new(1.5, egui::Color32::WHITE.linear_multiply(0.6))
                            );

                            if response.clicked() {
                                if is_code {
                                    self.app.pending_action = AppAction::SetViewMode(crate::app_state::ViewMode::PreviewOnly);
                                } else {
                                    self.app.pending_action = AppAction::SetViewMode(crate::app_state::ViewMode::CodeOnly);
                                }
                            }
                        }
                    });
                });
            });

        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if panel_resp.response.rect.expand(12.0).contains(pos) {
                keep_open = true;
            }
        }

        if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
            self.app.state.layout.show_tools_panel = false;
        }
    }
}
