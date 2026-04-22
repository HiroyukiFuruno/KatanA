use super::types::*;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::*;
use katana_platform::settings::types::RuleSeverity;

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

        Self::render_workspace_settings(ui, state, linter_msgs, &mut action);

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

        for meta in katana_linter::rules::markdown::eval::MarkdownLinterOps::get_official_rules() {
            let rule_id = meta.id().to_string();
            let current_severity = state
                .config
                .settings
                .settings()
                .linter
                .rule_severity
                .get(&rule_id)
                .copied()
                .unwrap_or_default();
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    ui.label(&rule_id);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        Self::render_severity_dropdown(
                            ui,
                            state,
                            msgs,
                            action,
                            &rule_id,
                            current_severity,
                        );
                    });
                })
                .show(ui);
        }
    }

    fn render_severity_dropdown(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
        rule_id: &str,
        current_severity: RuleSeverity,
    ) {
        let mut new_severity = current_severity;
        egui::ComboBox::from_id_source(format!("linter_sev_{}", rule_id))
            .selected_text(Self::severity_string(current_severity, msgs))
            .show_ui(ui, |ui| {
                if ui
                    .add(
                        egui::Button::selectable(
                            new_severity == RuleSeverity::Ignore,
                            &msgs.severity_ignore,
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    new_severity = RuleSeverity::Ignore;
                }
                if ui
                    .add(
                        egui::Button::selectable(
                            new_severity == RuleSeverity::Warning,
                            &msgs.severity_warning,
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    new_severity = RuleSeverity::Warning;
                }
                if ui
                    .add(
                        egui::Button::selectable(
                            new_severity == RuleSeverity::Error,
                            &msgs.severity_error,
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    new_severity = RuleSeverity::Error;
                }
            });

        if new_severity != current_severity {
            state
                .config
                .settings
                .settings_mut()
                .linter
                .rule_severity
                .insert(rule_id.to_string(), new_severity);
            let _ = state.config.try_save_settings();
            *action = Some(AppAction::RefreshDiagnostics);
        }
    }

    fn severity_string(sev: RuleSeverity, msgs: &LinterTranslations) -> String {
        match sev {
            RuleSeverity::Ignore => msgs.severity_ignore.clone(),
            RuleSeverity::Warning => msgs.severity_warning.clone(),
            RuleSeverity::Error => msgs.severity_error.clone(),
        }
    }

    fn render_workspace_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        if let Some(workspace_root) = state.workspace.data.as_ref().map(|w| w.root.clone()) {
            let json_path = workspace_root.join(".markdownlint.json");
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    if json_path.exists() {
                        ui.label(&msgs.workspace_has_config);
                        if ui
                            .add(egui::Button::new(&msgs.open_config).frame_when_inactive(true))
                            .clicked()
                        {
                            *action = Some(AppAction::SelectDocument(json_path));
                        }
                    } else {
                        ui.label(&msgs.workspace_no_config);
                        if ui
                            .add(egui::Button::new(&msgs.create_config).frame_when_inactive(true))
                            .clicked()
                            && std::fs::write(&json_path, "{\n  \"default\": true\n}\n").is_ok()
                        {
                            *action = Some(AppAction::SelectDocument(json_path));
                        }
                    }
                })
                .show(ui);
        } else {
            ui.label(&msgs.open_workspace_to_configure);
        }
    }
}
