use super::presets::LinterPresetOps;
use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct LinterPresetDialogOps;

const LINTER_SAVE_DIALOG_WIDTH: f32 = 400.0;
const LINTER_SAVE_INPUT_WIDTH: f32 = 260.0;

impl LinterPresetDialogOps {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        action: &mut Option<AppAction>,
    ) {
        let show_dialog = ui.data(|data| {
            data.get_temp::<bool>(egui::Id::new("katana_linter_saving_preset"))
                .unwrap_or(false)
        });
        if !show_dialog {
            return;
        }

        let mut preset_name = ui.data(|data| {
            data.get_temp::<String>(egui::Id::new("katana_linter_preset_name_input"))
                .unwrap_or_default()
        });
        let mut close_dialog = false;
        let mut saved = false;
        Self::render_window(ui, state, &mut preset_name, &mut close_dialog, &mut saved);
        if close_dialog {
            if saved {
                *action = Some(AppAction::RefreshDiagnostics);
            }
            Self::close_dialog(ui);
        } else {
            ui.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("katana_linter_preset_name_input"),
                    preset_name,
                )
            });
        }
    }

    fn render_window(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        preset_name: &mut String,
        close_dialog: &mut bool,
        saved: &mut bool,
    ) {
        let i18n = crate::i18n::I18nOps::get();
        egui::Window::new(&i18n.settings.icons.save_preset)
            .collapsible(false)
            .resizable(false)
            .min_width(LINTER_SAVE_DIALOG_WIDTH)
            .max_width(LINTER_SAVE_DIALOG_WIDTH)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        ui.label(&i18n.settings.icons.preset_name);
                        ui.add(
                            egui::TextEdit::singleline(preset_name)
                                .desired_width(LINTER_SAVE_INPUT_WIDTH),
                        )
                        .request_focus();
                    })
                    .show(ui);
                ui.add_space(crate::settings::SUBSECTION_SPACING);
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        Self::render_buttons(ui, state, preset_name, close_dialog, saved);
                    })
                    .show(ui);
            });
    }

    fn render_buttons(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        preset_name: &str,
        close_dialog: &mut bool,
        saved: &mut bool,
    ) {
        let i18n = crate::i18n::I18nOps::get();
        if ui.button(&i18n.action.cancel).clicked() {
            *close_dialog = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(&i18n.action.save).clicked() && !preset_name.is_empty() {
                LinterPresetOps::save_current_as_user_preset(
                    &mut state.config.settings.settings_mut().linter,
                    preset_name,
                );
                let _ = state.config.try_save_settings();
                *saved = true;
                *close_dialog = true;
            }
        });
    }

    fn close_dialog(ui: &mut egui::Ui) {
        ui.data_mut(|data| {
            data.insert_temp(egui::Id::new("katana_linter_saving_preset"), false);
            data.insert_temp(
                egui::Id::new("katana_linter_preset_name_input"),
                String::new(),
            );
        });
    }
}
