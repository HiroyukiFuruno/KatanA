use super::types::*;
use crate::app_state::AppAction;
use crate::settings::*;
use crate::widgets::StyledComboBox;

impl UpdatesTabOps {
    pub(crate) fn render_updates_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        let update_msgs = &crate::i18n::I18nOps::get().settings.updates;

        SettingsOps::section_header(ui, &update_msgs.section_title);

        let ver_str = format!("Current version: v{}", env!("CARGO_PKG_VERSION"));
        ui.label(egui::RichText::new(ver_str).weak().size(HINT_FONT_SIZE));

        // WHY: allow(horizontal_layout)
        crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
            ui.label(&update_msgs.interval);

            let mut interval = state.config.settings.settings().updates.interval;
            use katana_platform::settings::UpdateInterval;
            let mut changed = false;

            StyledComboBox::new(
                "update_interval",
                match interval {
                    UpdateInterval::Never => update_msgs.never.as_str(),
                    UpdateInterval::Daily => update_msgs.daily.as_str(),
                    UpdateInterval::Weekly => update_msgs.weekly.as_str(),
                    UpdateInterval::Monthly => update_msgs.monthly.as_str(),
                },
            )
            .show(ui, |ui| {
                // WHY: in popup/list context; future: standardize as atom
                if ui.add(egui::Button::selectable(interval == UpdateInterval::Never, &update_msgs.never).frame_when_inactive(true)).clicked() {
                    interval = UpdateInterval::Never;
                    changed = true;
                }
                // WHY: in popup/list context; future: standardize as atom
                if ui.add(egui::Button::selectable(interval == UpdateInterval::Daily, &update_msgs.daily).frame_when_inactive(true)).clicked() {
                    interval = UpdateInterval::Daily;
                    changed = true;
                }
                // WHY: in popup/list context; future: standardize as atom
                if ui.add(egui::Button::selectable(interval == UpdateInterval::Weekly, &update_msgs.weekly).frame_when_inactive(true)).clicked() {
                    interval = UpdateInterval::Weekly;
                    changed = true;
                }
                // WHY: in popup/list context; future: standardize as atom
                if ui.add(egui::Button::selectable(interval == UpdateInterval::Monthly, &update_msgs.monthly).frame_when_inactive(true)).clicked() {
                    interval = UpdateInterval::Monthly;
                    changed = true;
                }
            });

            if changed {
                state.config.settings.settings_mut().updates.interval = interval;
                let _ = state.config.try_save_settings();
            }
        }).show(ui);

        ui.add_space(SUBSECTION_SPACING);

        if ui.button(&update_msgs.check_now).clicked() {
            return Some(AppAction::CheckForUpdates);
        }
        None
    }
}
