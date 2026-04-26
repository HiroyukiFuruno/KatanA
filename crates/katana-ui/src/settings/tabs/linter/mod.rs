use super::types::*;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::*;

mod advanced;
mod preset_controls;
mod preset_dialog;
mod presets;
mod properties;
mod properties_helpers;
mod rule_group;
mod rule_severity;
mod severity_toggle;
mod workspace;

use workspace::WorkspaceSettingsOps;

impl LinterTabOps {
    pub(crate) fn render_linter_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        let mut action: Option<AppAction> = None;
        let linter_msgs = &crate::i18n::I18nOps::get().linter;
        let mut is_advanced_open = ui
            .data(|data| data.get_temp::<bool>(egui::Id::new("linter_advanced_is_open")))
            .unwrap_or(false);
        preset_dialog::LinterPresetDialogOps::render(ui, state, &mut action);

        if is_advanced_open {
            advanced::LinterAdvancedSettingsOps::render_advanced_settings(
                ui,
                state,
                linter_msgs,
                &mut is_advanced_open,
            );
        } else {
            Self::render_normal_linter_tab(
                ui,
                state,
                linter_msgs,
                &mut action,
                &mut is_advanced_open,
            );
        }

        ui.data_mut(|data| {
            data.insert_temp(egui::Id::new("linter_advanced_is_open"), is_advanced_open)
        });
        Self::consume_pending_linter_update(ui, &mut action);
        action
    }

    fn render_normal_linter_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        linter_msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
        is_advanced_open: &mut bool,
    ) {
        let enabled = Self::render_enabled_toggle(ui, state, linter_msgs, action);
        ui.add_space(SUBSECTION_SPACING);
        if !enabled {
            return;
        }

        preset_controls::LinterPresetControlsOps::render(ui, state, action, is_advanced_open);
        ui.add_space(SUBSECTION_SPACING);

        let mut force_open: Option<bool> = None;
        rule_severity::LinterRuleSeverityOps::render_expand_controls(ui, &mut force_open);
        ui.add_space(SUBSECTION_SPACING);
        rule_severity::LinterRuleSeverityOps::render_rule_severities(
            ui,
            state,
            linter_msgs,
            action,
            force_open,
        );
        ui.add_space(SUBSECTION_SPACING);
        WorkspaceSettingsOps::render_workspace_settings(
            ui,
            state,
            linter_msgs,
            is_advanced_open,
            force_open,
        );
    }

    fn render_enabled_toggle(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        linter_msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) -> bool {
        let mut enabled = state.config.settings.settings().linter.enabled;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(&linter_msgs.enable_linter, &mut enabled)
                    .position(crate::widgets::TogglePosition::Right)
                    .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state.config.settings.settings_mut().linter.enabled = enabled;
            let _ = state.config.try_save_settings();
            *action = Some(AppAction::RefreshDiagnostics);
        }
        enabled
    }

    fn consume_pending_linter_update(ui: &mut egui::Ui, action: &mut Option<AppAction>) {
        let pending = ui.data(|data| {
            data.get_temp::<bool>(egui::Id::new("katana_pending_linter_update"))
                .unwrap_or(false)
        });
        if !pending {
            return;
        }
        ui.data_mut(|data| data.insert_temp(egui::Id::new("katana_pending_linter_update"), false));
        *action = Some(AppAction::RefreshDiagnostics);
    }
}
