use eframe::egui;

pub(super) struct RulePropertyComboboxOps;

impl RulePropertyComboboxOps {
    pub(super) fn render(
        ui: &mut egui::Ui,
        combo_id: &str,
        opts: &[&str],
        current_val: &mut String,
    ) -> bool {
        let mut changed = false;
        const COMBO_BOX_WIDTH: f32 = 120.0;

        crate::widgets::StyledComboBox::new(combo_id, Self::format_label(current_val.as_str()))
            .width(COMBO_BOX_WIDTH)
            .show(ui, |ui| {
                for opt in opts {
                    let is_selected = current_val == opt;
                    if ui
                        .add(
                            egui::Button::selectable(is_selected, Self::format_label(opt))
                                .frame_when_inactive(true),
                        )
                        .clicked()
                    {
                        *current_val = (*opt).to_string();
                        changed = true;
                    }
                }
            });
        changed
    }

    fn format_label(value: &str) -> String {
        let value = value.replace('_', " ");
        let mut chars = value.chars();
        match chars.next() {
            None => String::new(),
            Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}
