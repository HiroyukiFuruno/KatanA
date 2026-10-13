use super::side_panels::{PREVIEW_SIDE_BAR_MARGIN, PREVIEW_SIDE_BAR_WIDTH, PreviewSidePanels};
use eframe::egui;

impl PreviewSidePanels<'_> {
    pub(super) fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        let availability = self.sidebar_availability();
        let panel_response = egui::Panel::right("preview_side_bar")
            .resizable(false)
            .exact_size(PREVIEW_SIDE_BAR_WIDTH)
            .show_inside(ui, |ui| {
                ui.add_space(PREVIEW_SIDE_BAR_MARGIN);
                ui.vertical_centered(|ui| {
                    self.render_sidebar_controls(ui, availability);
                });
            });
        self.sidebar_rect = Some(panel_response.response.rect);
        ui.painter().line_segment(
            [
                panel_response.response.rect.left_top(),
                panel_response.response.rect.left_bottom(),
            ],
            ui.visuals().window_stroke(),
        );
    }
}
