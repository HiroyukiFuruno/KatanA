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
            /* WHY: Nudge 2px up to visually align with text baseline */
            offset_y: COLOR_OFFSET_Y,
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
        self.show_color_button(ui, color, egui::color_picker::Alpha::Opaque)
    }

    pub fn show_rgba(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        self.show_color_button(ui, color, egui::color_picker::Alpha::BlendOrAdditive)
    }

    fn show_color_button(
        self,
        ui: &mut egui::Ui,
        color: &mut egui::Color32,
        alpha: egui::color_picker::Alpha,
    ) -> egui::Response {
        let row_height = COLOR_ROW_HEIGHT;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.add_space(COLOR_LABEL_MARGIN);
                ui.add_sized(
                    [self.label_width, row_height],
                    egui::Label::new(self.label).truncate(),
                );
                ui.add_space(self.spacing);
                let button_size = ui.spacing().interact_size;
                let (button_rect, _) = ui.allocate_exact_size(button_size, egui::Sense::hover());
                let max_rect = button_rect.translate(egui::vec2(0.0, self.offset_y));
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(max_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| egui::color_picker::color_edit_button_srgba(ui, color, alpha),
                )
                .inner
            },
        )
        .inner
    }
}

impl InlineColorPicker {
    pub fn new() -> Self {
        Self { is_rgba: false }
    }

    pub fn rgba(mut self, is_rgba: bool) -> Self {
        self.is_rgba = is_rgba;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, color: &mut egui::Color32) -> egui::Response {
        let alpha = if self.is_rgba {
            egui::color_picker::Alpha::BlendOrAdditive
        } else {
            egui::color_picker::Alpha::Opaque
        };

        egui::color_picker::color_edit_button_srgba(ui, color, alpha)
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
