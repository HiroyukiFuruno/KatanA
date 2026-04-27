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
        const COMBO_BOX_WIDTH: f32 = 180.0;

        let selected_label = Self::format_label(current_val.as_str());
        crate::widgets::StyledComboBox::new(combo_id, &selected_label)
            .width(COMBO_BOX_WIDTH)
            .show(ui, |ui| {
                for opt in opts {
                    let is_selected = current_val == opt;
                    let option_label = Self::format_label(opt);
                    if ui
                        .add(
                            egui::Button::selectable(is_selected, &option_label)
                                .frame_when_inactive(true),
                        )
                        .on_hover_text(&option_label)
                        .clicked()
                    {
                        *current_val = (*opt).to_string();
                        changed = true;
                    }
                }
            })
            .on_hover_text(selected_label);
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
