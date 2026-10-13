use crate::app_state::AppState;
use crate::i18n::SettingsUpdateMessages;
use eframe::egui;
use katana_platform::settings::UpdateInterval;

pub(crate) struct UpdateIntervalSelector;

impl UpdateIntervalSelector {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        state: &mut AppState,
        i18n_update: &SettingsUpdateMessages,
    ) {
        let mut interval = state.config.settings.settings().updates.interval;
        let mut changed = false;

        let selected_label = interval.as_pulldown_label().to_string();
        let selected_label_for_hover = selected_label.clone();
        crate::widgets::AlignCenter::new()
            .left(|ui| ui.label(&i18n_update.interval))
            .right(|ui| {
                crate::widgets::StyledComboBox::new("update_check_interval", selected_label)
                    .show(ui, |ui| {
                        for candidate in UpdateInterval::ui_options() {
                            let candidate = *candidate;
                            let is_selected = candidate == interval;
                            let label = candidate.as_pulldown_label().to_string();
                            if ui
                                .add(
                                    egui::Button::selectable(is_selected, label)
                                        .frame_when_inactive(true),
                                )
                                .clicked()
                            {
                                interval = candidate;
                                changed = true;
                            }
                        }
                    })
                    .on_hover_text(selected_label_for_hover)
            })
            .show(ui);

        if changed {
            state.config.settings.settings_mut().updates.interval = interval;
            let _ = state.config.try_save_settings();
        }
    }
}
