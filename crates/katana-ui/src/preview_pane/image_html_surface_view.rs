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

    fn show_error(&self, ui: &mut egui::Ui, rect: egui::Rect, error: &str) {
        let content_rect = rect.shrink(HTML_BROWSER_ERROR_TEXT_PADDING);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.set_max_width(content_rect.width());
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(error)
                                    .monospace()
                                    .color(ui.visuals().error_fg_color),
                            )
                            .wrap(),
                        );
                    });
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview_pane::image_html_surface::frame::BrowserFrame;

    #[test]
    fn rendered_browser_frame_keeps_the_host_input_surface_hoverable() {
        let context = egui::Context::default();
        let viewport =
            katana_document_viewer::browser_session::HtmlBrowserViewport::new(100, 60, 1.0)
                .expect("viewport");
        let mut surface = HtmlBrowserSurface::failed("unused".to_owned());
        surface.error = None;
        surface.frame = Some(BrowserFrame::new(
            1,
            viewport,
            0.0,
            120.0,
            vec![255; 100 * 60 * 4],
        ));

        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            surface.show(ui);
        });
        output.textures_delta.clear();
        let mut input = egui::RawInput::default();
        input
            .events
            .push(egui::Event::PointerMoved(egui::pos2(20.0, 20.0)));
        let mut output = context.run_ui(input, |ui| {
            surface.show(ui);
        });
        output.textures_delta.clear();

        assert!(surface.pointer_over);
    }
}
