use super::side_panels::{
    PreviewSidePanels, PANEL_ANIM_SPEED, PANEL_HEAD_SPACE, PANEL_HOVER_MARGIN, PANEL_WIDTH,
};
use crate::app_state::AppAction;
use eframe::egui;

const EXPORT_BTN_H: f32 = 32.0;
const EXPORT_ITEM_SPACE: f32 = 4.0;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_export(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("export_panel_anim"),
            self.app.state.layout.show_export_panel,
            PANEL_ANIM_SPEED,
        );

        if anim == 0.0 {
            return;
        }

        let mut keep_open = false;
        /* WHY: Keep panel open while pointer is near the toggle button. */
        if self.app.state.layout.show_export_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            && let Some(btn_rect) = self.export_btn_rect
            && btn_rect.expand(PANEL_HOVER_MARGIN).contains(pos)
        {
            keep_open = true;
        }

        let panel_resp = egui::SidePanel::right("preview_export_panel")
            .resizable(false)
            .exact_width(PANEL_WIDTH * anim)
            .show_inside(ui, |ui| {
                ui.set_min_width(PANEL_WIDTH);
                ui.vertical(|ui| {
                    ui.add_space(PANEL_HEAD_SPACE);

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
                        ui.spacing_mut().item_spacing.y = EXPORT_ITEM_SPACE;
                        for (icon, label, fmt) in formats {
                            let resp = ui.add(
                                egui::Button::image_and_text(
                                    icon.ui_image(ui, crate::icon::IconSize::Medium),
                                    label,
                                )
                                .fill(crate::theme_bridge::TRANSPARENT)
                                .min_size(egui::vec2(ui.available_width(), EXPORT_BTN_H)),
                            );
                            if resp.clicked() {
                                self.app.pending_action = AppAction::ExportDocument(fmt);
                            }
                        }
                    });
                });
            });

        if self.app.state.layout.show_export_panel {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                && panel_resp.response.rect.expand(PANEL_HOVER_MARGIN).contains(pos)
            {
                keep_open = true;
            }

            if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
                self.app.state.layout.show_export_panel = false;
            }
        }
    }
}
