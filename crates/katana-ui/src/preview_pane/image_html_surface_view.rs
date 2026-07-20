use super::{HTML_BROWSER_ERROR_TEXT_PADDING, HtmlBrowserSurface, frame_display_size};
use eframe::egui::{self, Vec2};

impl HtmlBrowserSurface {
    pub(super) fn show(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        self.resize_to_ui(ui);
        let size = self.display_size(ui);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        self.last_display_rect = Some(rect);
        self.pointer_over = response.hovered();
        self.forward_input(ui, rect, &response);

        if let Some(texture) = &self.texture {
            ui.put(
                rect,
                egui::Image::new(texture).fit_to_exact_size(rect.size()),
            );
        } else if let Some(error) = &self.error {
            self.show_error(ui, rect, error);
        } else {
            self.show_loading(ui, rect);
        }
        rect
    }

    fn display_size(&self, ui: &egui::Ui) -> Vec2 {
        self.frame
            .as_ref()
            .map(|frame| frame_display_size(frame.viewport))
            .unwrap_or_else(|| ui.available_size().max(Vec2::splat(1.0)))
    }

    fn show_error(&self, ui: &egui::Ui, rect: egui::Rect, error: &str) {
        ui.painter().text(
            rect.min
                + egui::vec2(
                    HTML_BROWSER_ERROR_TEXT_PADDING,
                    HTML_BROWSER_ERROR_TEXT_PADDING,
                ),
            egui::Align2::LEFT_TOP,
            format!("HTML browser error: {error}"),
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().error_fg_color,
        );
    }

    fn show_loading(&self, ui: &egui::Ui, rect: egui::Rect) {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Loading HTML browser...",
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().weak_text_color(),
        );
    }
}
