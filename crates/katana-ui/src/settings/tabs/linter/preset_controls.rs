use super::presets::LinterPresetOps;
use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct LinterPresetControlsOps;

impl LinterPresetControlsOps {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        action: &mut Option<AppAction>,
        is_advanced_open: &mut bool,
    ) {
        let i18n = crate::i18n::I18nOps::get();
        let linter_settings = state.config.settings.settings().linter.clone();
        let built_in_presets = LinterPresetOps::built_in_presets(&i18n.linter);
        let user_presets = LinterPresetOps::user_preset_references(&linter_settings);
        let mut preset_state = linter_settings.preset_state.clone();
        preset_state.sync_user_preset_names(user_presets.iter().map(|preset| &preset.label));

        let labels = crate::settings::tabs::preset_widget::PresetWidgetLabels {
            title: &i18n.linter.preset_label,
            save: &i18n.settings.icons.save_preset,
            revert: &i18n.settings.icons.revert_default,
            advanced: Some(&i18n.linter.rule_details),
        };
        let response = crate::settings::tabs::preset_widget::PresetWidgetOps::render(
            ui,
            "linter_preset_widget",
            &preset_state,
            &built_in_presets,
            &user_presets,
            labels,
        );
        Self::handle_response(state, ui, response, action, is_advanced_open);
    }

    fn handle_response(
        state: &mut crate::app_state::AppState,
        ui: &mut egui::Ui,
        response: crate::settings::tabs::preset_widget::PresetWidgetResponse,
        action: &mut Option<AppAction>,
        is_advanced_open: &mut bool,
    ) {
        let mut changed = false;
        if let Some(reference) = response.selected {
            changed = LinterPresetOps::apply_reference(
                &mut state.config.settings.settings_mut().linter,
                &reference,
            );
        }
        if response.revert_clicked {
            changed = LinterPresetOps::apply_base(&mut state.config.settings.settings_mut().linter);
        }
        if response.save_clicked {
            LinterPresetOps::sync_user_preset_state(
                &mut state.config.settings.settings_mut().linter,
            );
            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("katana_linter_saving_preset"), true);
            });
        }
        if response.advanced_clicked {
            *is_advanced_open = true;
        }
        if changed {
            let _ = state.config.try_save_settings();
            *action = Some(AppAction::RefreshDiagnostics);
        }
    }
}
