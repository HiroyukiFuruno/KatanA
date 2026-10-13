mod update_interval_selector;

use crate::app_action::AppAction;
use crate::state::update::UpdatePhase;
use eframe::egui;

const SECTION_SPACING: f32 = 8.0;

impl crate::settings::tabs::UpdatesTabOps {
    pub(crate) fn render_updates_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        let mut pending_action = None;
        let i18n_root = crate::i18n::I18nOps::get();
        let i18n_update = &i18n_root.update;
        let i18n_settings = &i18n_root.settings.updates;

        ui.vertical(|ui| {
            /* 1. App Updates Section */
            ui.heading(&i18n_settings.section_title);
            ui.add_space(SECTION_SPACING);
            update_interval_selector::UpdateIntervalSelector::render(ui, state, i18n_settings);
            ui.add_space(SECTION_SPACING);

            let current_version = env!("CARGO_PKG_VERSION");
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    ui.label(&i18n_root.about.version);
                    ui.strong(current_version);
                })
                .show(ui);

            if let Some(update) = &state.update.available {
                ui.add_space(SECTION_SPACING);
                ui.label(&i18n_update.update_available);
                ui.strong(&update.tag_name);

                match &state.update.phase {
                    Some(UpdatePhase::Downloading { .. }) => {
                        ui.add_space(SECTION_SPACING);
                        crate::widgets::AlignCenter::new()
                            .content(|ui| {
                                ui.spinner();
                                ui.label(&i18n_update.downloading);
                            })
                            .show(ui);
                    }
                    Some(UpdatePhase::ReadyToRelaunch) => {
                        ui.add_space(SECTION_SPACING);
                        if ui.button(&i18n_update.install_update).clicked() {
                            pending_action = Some(AppAction::InstallUpdateAndRestart);
                        }
                    }
                    _ => {
                        ui.add_space(SECTION_SPACING);
                        if ui.button(&i18n_update.download_update).clicked() {
                            pending_action = Some(AppAction::StartUpdateDownload);
                        }
                    }
                }
            } else if state.update.checking {
                ui.add_space(SECTION_SPACING);
                crate::widgets::AlignCenter::new()
                    .content(|ui| {
                        ui.spinner();
                        ui.label(&i18n_update.checking_for_updates);
                    })
                    .show(ui);
            } else {
                ui.add_space(SECTION_SPACING);
                ui.label(&i18n_update.up_to_date);
                if ui.button(&i18n_settings.check_now).clicked() {
                    pending_action = Some(AppAction::CheckForUpdates);
                }
            }
        });

        pending_action
    }
}
