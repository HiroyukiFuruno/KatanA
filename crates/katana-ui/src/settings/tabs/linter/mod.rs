use super::types::*;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::*;
mod severity_toggle;
mod workspace;
use severity_toggle::SEVERITY_SEGMENT_HEIGHT;
use workspace::WorkspaceSettingsOps;
const RULE_DESC_SPACING: f32 = 8.0;
impl LinterTabOps {
    pub(crate) fn render_linter_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        let mut action: Option<AppAction> = None;
        let linter_msgs = &crate::i18n::I18nOps::get().linter;

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
            action = Some(AppAction::RefreshDiagnostics);
        }

        ui.add_space(SUBSECTION_SPACING);

        if enabled {
            Self::render_rule_severities(ui, state, linter_msgs, &mut action);
        }

        ui.add_space(SUBSECTION_SPACING);
        ui.label(egui::RichText::new(&linter_msgs.advanced_workspace_settings).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        WorkspaceSettingsOps::render_workspace_settings(ui, state, linter_msgs, &mut action);

        action
    }

    fn render_rule_severities(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        ui.label(egui::RichText::new(&msgs.rule_severities).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        /* WHY: get_user_configurable_rules() filters out internal-only rules (e.g. BrokenLinkRule)
         * and returns them sorted by ID for consistent presentation. */
        for rule in
            katana_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
        {
            let rule_id = rule.id().to_string();
            let current_severity = state
                .config
                .settings
                .settings()
                .linter
                .rule_severity
                .get(&rule_id)
                .copied()
                .unwrap_or_default();

            /* WHY: Use horizontal layout to pin the label left and the toggle right
             * within the available width, preventing overflow. */
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), SEVERITY_SEGMENT_HEIGHT),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    let new_severity =
                        severity_toggle::SeverityToggleOps::render_severity_segment_toggle(
                            ui,
                            &format!("lsev_{}", rule_id),
                            current_severity,
                            msgs,
                        );

                    if new_severity != current_severity {
                        state
                            .config
                            .settings
                            .settings_mut()
                            .linter
                            .rule_severity
                            .insert(rule_id.clone(), new_severity);
                        let _ = state.config.try_save_settings();
                        *action = Some(AppAction::RefreshDiagnostics);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(egui::Label::new(&rule_id).truncate());

                        let desc_key = rule_id.to_lowercase();
                        let description = msgs
                            .rule_descriptions
                            .get(&desc_key)
                            .map(|s| s.as_str())
                            .or_else(|| rule.official_meta().map(|m| m.description))
                            .unwrap_or("");

                        if !description.is_empty() {
                            ui.add_space(RULE_DESC_SPACING);
                            ui.label(egui::RichText::new(description).weak().small());
                        }
                    });
                },
            );

            ui.add_space(2.0);
        }
    }
}
