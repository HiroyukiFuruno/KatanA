use super::types::*;

impl<'a> StyledComboBox<'a> {
    pub fn new(id: &'a str, selected_text: impl Into<String>) -> Self {
        Self {
            id,
            selected_text: selected_text.into(),
            width: None,
            truncate: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        ui.scope(|ui| {
            Self::apply_popup_visuals(ui);
            self.show_inner(ui, content)
        })
        .inner
    }

    fn show_inner(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let mut combo = egui::ComboBox::from_id_salt(self.id).selected_text(self.selected_text);

        if let Some(width) = self.width {
            combo = combo.width(width);
        }
        if self.truncate {
            combo = combo.truncate();
        }

        combo
            .show_ui(ui, |ui| {
                Self::apply_popup_visuals(ui);
                content(ui);
            })
            .response
    }

    fn apply_popup_visuals(ui: &mut egui::Ui) {
        let visuals = &mut ui.style_mut().visuals;
        visuals.widgets.inactive.bg_fill = crate::theme_bridge::TRANSPARENT;
        visuals.widgets.inactive.weak_bg_fill = crate::theme_bridge::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_combobox_builder_defaults() {
        let combo = StyledComboBox::new("test_id", "Selected");
        assert_eq!(combo.id, "test_id");
        assert_eq!(combo.selected_text, "Selected");
        assert!(combo.width.is_none());
    }

    #[test]
    fn test_styled_combobox_builder_with_width() {
        let combo = StyledComboBox::new("test_id", "Selected").width(150.0);
        assert_eq!(combo.width, Some(150.0));
    }

    #[test]
    fn test_styled_combobox_renders_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                StyledComboBox::new("render_test", "Value").show(ui, |ui| {
                    ui.label("item_a");
                });
            });
        });
    }

    #[test]
    fn test_styled_combobox_renders_with_width() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                StyledComboBox::new("width_test", "Value")
                    .width(200.0)
                    .show(ui, |ui| {
                        ui.label("item_a");
                    });
            });
        });
    }

    #[test]
    fn test_styled_combobox_popup_visuals_make_inactive_rows_transparent() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                StyledComboBox::apply_popup_visuals(ui);

                assert_eq!(
                    ui.visuals().widgets.inactive.weak_bg_fill,
                    crate::theme_bridge::TRANSPARENT
                );
                assert_eq!(
                    ui.visuals().widgets.inactive.bg_fill,
                    crate::theme_bridge::TRANSPARENT
                );
            });
        });
    }
}
