use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_ui::{LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG};
use eframe::egui;

pub struct PreviewSidePanels<'a> {
    pub app: &'a mut KatanaApp,
}

impl<'a> PreviewSidePanels<'a> {
    pub fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let doc_exists = self.app.state.active_document().is_some();
        if !doc_exists {
            return;
        }

        self.render_side_bar(ui);
        self.render_toc(ui);
        self.render_export(ui);
        self.render_story(ui);
    }

    fn render_side_bar(&mut self, ui: &mut egui::Ui) {
        use katana_platform::settings::TocPosition;
        let position = self
            .app
            .state
            .config
            .settings
            .settings()
            .layout
            .toc_position;

        let panel = match position {
            TocPosition::Left => egui::SidePanel::left("preview_side_bar"),
            TocPosition::Right => egui::SidePanel::right("preview_side_bar"),
        };

        panel
            .exact_width(40.0)
            .resizable(false)
            .frame(egui::Frame::side_top_panel(&ui.ctx().global_style()).inner_margin(4.0))
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);

                    self.render_toggle_button(
                        ui,
                        crate::Icon::Toc,
                        self.app.state.layout.show_toc,
                        AppAction::ToggleToc,
                        "TOC",
                    );
                    ui.add_space(4.0);
                    self.render_toggle_button(
                        ui,
                        crate::Icon::Export,
                        self.app.state.layout.show_export_panel,
                        AppAction::ToggleExportPanel,
                        "Export",
                    );
                    ui.add_space(4.0);
                    self.render_toggle_button(
                        ui,
                        crate::Icon::Preview,
                        self.app.state.layout.show_story_panel,
                        AppAction::ToggleStoryPanel,
                        "Story",
                    );

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        self.render_back_to_top(ui);
                    });
                });
            });
    }

    fn render_toggle_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: crate::Icon,
        is_active: bool,
        action: AppAction,
        tooltip: &str,
    ) {
        let icon_bg = if ui.visuals().dark_mode {
            egui::Color32::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG)
        };

        let active_bg = if ui.visuals().dark_mode {
            ui.visuals().selection.bg_fill
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
        };

        let bg = if is_active { active_bg } else { icon_bg };

        let resp = ui
            .add(
                egui::Button::image(icon.ui_image(ui, crate::icon::IconSize::Medium))
                    .fill(bg)
                    .min_size(egui::vec2(32.0, 32.0))
                    .rounding(egui::Rounding::same(4)),
            )
            .on_hover_text(tooltip);

        if resp.clicked() {
            self.app.pending_action = action;
        }
    }

    fn render_back_to_top(&mut self, ui: &mut egui::Ui) {
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
            // Check scroll position from mapper if available, or just show if content is long
            // Actually, we can check the scroll offset from the AppState if it's synced.
            let scroll_offset = self.app.state.scroll.preview_y;
            if scroll_offset > 400.0 {
                let resp = ui
                    .add(
                        egui::Button::image(
                            crate::Icon::ArrowUp.ui_image(ui, crate::icon::IconSize::Medium),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(32.0, 32.0)),
                    )
                    .on_hover_text("Back to Top");

                if resp.clicked() {
                    preview.pane.scroll_request = Some(0);
                }
            }
        }
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

        egui::SidePanel::right("preview_export_panel")
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
    }

    fn render_story(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_story_panel {
            return;
        }

        egui::SidePanel::right("preview_story_panel")
            .resizable(true)
            .default_width(260.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.heading(
                            crate::i18n::I18nOps::get()
                                .preview
                                .slideshow_settings
                                .clone(),
                        );
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;

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

                        ui.add_space(16.0);

                        let mut hover = self
                            .app
                            .state
                            .config
                            .settings
                            .settings()
                            .behavior
                            .slideshow_hover_highlight;
                        if ui
                            .checkbox(
                                &mut hover,
                                &crate::i18n::I18nOps::get().preview.highlight_hover,
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
                            .checkbox(
                                &mut controls,
                                &crate::i18n::I18nOps::get().preview.show_diagram_controls,
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
                    });
                });
            });
    }
}
