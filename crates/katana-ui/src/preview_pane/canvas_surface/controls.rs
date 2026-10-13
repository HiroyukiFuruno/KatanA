use eframe::egui;

use super::{CanvasSurface, CanvasSurfaceLabels, geometry::CanvasLayout};
use crate::icon::{Icon, IconSize};

const ZOOM_STEP: f32 = 1.25;

impl CanvasSurface {
    pub(super) fn controls_ui(&mut self, ui: &mut egui::Ui, labels: &CanvasSurfaceLabels<'_>) {
        let messages = crate::i18n::I18nOps::get();
        let controller = &messages.preview.diagram_controller;
        let viewport = ui.available_size();
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                if Self::control_ui(ui, Icon::ZoomOut, &controller.zoom_out) {
                    self.view_rect =
                        CanvasLayout::zoom_view(self.view_rect, viewport, ZOOM_STEP.recip());
                }
                if Self::control_ui(ui, Icon::ZoomIn, &controller.zoom_in) {
                    self.view_rect = CanvasLayout::zoom_view(self.view_rect, viewport, ZOOM_STEP);
                }
                if Self::control_ui(ui, Icon::ResetView, labels.fit) {
                    self.reset_view();
                }
                ui.label(&controller.trackpad_help);
            })
            .show(ui);
    }

    fn control_ui(ui: &mut egui::Ui, icon: Icon, label: &str) -> bool {
        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg()
        };
        let response =
            ui.add(egui::Button::image(icon.image(IconSize::Small).alt_text(label)).fill(icon_bg));
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
        });
        response.on_hover_text(label).clicked()
    }
}
