use super::types::*;

pub const COLOR_LABEL_WIDTH: f32 = 130.0;
pub const COLOR_SPACING: f32 = 16.0;
pub const COLOR_OFFSET_Y: f32 = -2.0;
pub const COLOR_ROW_HEIGHT: f32 = 24.0;
pub const COLOR_LABEL_MARGIN: f32 = 8.0;

impl<'a> LabeledColorPicker<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            label_width: COLOR_LABEL_WIDTH,
            spacing: COLOR_SPACING,
            offset_y: COLOR_OFFSET_Y, /* WHY: Nudge 2px up to visually align with text baseline */
            is_rgba: false,
        }
    }

    pub fn rgba(mut self, is_rgba: bool) -> Self {
        self.is_rgba = is_rgba;
        self
    }

    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn offset_y(mut self, offset: f32) -> Self {
        self.offset_y = offset;
        self
    }

    pub fn show_rgb(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        let available_w = ui.available_width();
        let row_height = COLOR_ROW_HEIGHT; /* WHY: Standardize row height for strict table alignment */
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available_w, row_height), egui::Sense::hover());

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.allocate_exact_size(egui::vec2(0.0, rect.height()), egui::Sense::hover());
                ui.add_space(COLOR_LABEL_MARGIN);
                ui.label(self.label);
            });
        });

        let right_rect = rect.translate(egui::vec2(0.0, self.offset_y));
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_exact_size(egui::vec2(0.0, right_rect.height()), egui::Sense::hover());
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    color,
                    egui::color_picker::Alpha::Opaque,
                )
            })
            .inner
        })
        .inner
    }

    pub fn show_rgba(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        let available_w = ui.available_width();
        let row_height = COLOR_ROW_HEIGHT;
        let (rect, _response) =
            ui.allocate_exact_size(egui::vec2(available_w, row_height), egui::Sense::hover());

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.allocate_exact_size(egui::vec2(0.0, rect.height()), egui::Sense::hover());
                ui.add_space(COLOR_LABEL_MARGIN);
                ui.label(self.label);
            });
        });

        let right_rect = rect.translate(egui::vec2(0.0, self.offset_y));
        ui.scope_builder(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_exact_size(egui::vec2(0.0, right_rect.height()), egui::Sense::hover());
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    color,
                    egui::color_picker::Alpha::BlendOrAdditive,
                )
            })
            .inner
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labeled_color_picker_defaults() {
        let picker = LabeledColorPicker::new("Test Label");
        assert_eq!(picker.label, "Test Label");
        assert_eq!(picker.offset_y, COLOR_OFFSET_Y);
        assert_eq!(picker.label_width, COLOR_LABEL_WIDTH);
        assert_eq!(picker.spacing, COLOR_SPACING);
    }

    #[test]
    fn test_labeled_color_picker_customization() {
        let picker = LabeledColorPicker::new("Custom")
            .offset_y(10.0)
            .label_width(200.0)
            .spacing(5.0)
            .rgba(true);
        assert_eq!(picker.offset_y, 10.0);
        assert_eq!(picker.label_width, 200.0);
        assert_eq!(picker.spacing, 5.0);
        assert!(picker.is_rgba);
    }
}
