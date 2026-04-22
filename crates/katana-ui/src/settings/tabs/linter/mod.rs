use super::types::*;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::*;
mod severity_toggle;
use severity_toggle::{SEVERITY_SEGMENT_HEIGHT, SEVERITY_TOGGLE_WIDTH};
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
            crate::widgets::AlignCenter::new().content(|ui| {
                let toggle_width = SEVERITY_TOGGLE_WIDTH;
                let available = ui.available_width();
                let label_width = (available - toggle_width - ui.spacing().item_spacing.x).max(0.0);

                /* WHY: Truncate label to the remaining space so wide rule IDs never push
                 * the severity toggle off-screen. */
                ui.add_sized(
                    [label_width, SEVERITY_SEGMENT_HEIGHT],
                    egui::Label::new(&rule_id).truncate(),
                );

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
                        .insert(rule_id, new_severity);
                    let _ = state.config.try_save_settings();
                    *action = Some(AppAction::RefreshDiagnostics);
                }
            });

            ui.add_space(2.0);
        }
    }

    fn render_workspace_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        let mut use_workspace = state
            .config
            .settings
            .settings()
            .linter
            .use_workspace_local_config;

        let global_config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("KatanA");
        let global_json_path = global_config_dir.join(".markdownlint.json");

        let workspace_json_path = state
            .workspace
            .data
            .as_ref()
            .map(|w| w.root.join(".markdownlint.json"));

        if let Some(ws_path) = &workspace_json_path {
            if ui
                .add(
                    crate::widgets::LabeledToggle::new(
                        &msgs.use_workspace_local_config,
                        &mut use_workspace,
                    )
                    .position(crate::widgets::TogglePosition::Right)
                    .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                )
                .changed()
            {
                state
                    .config
                    .settings
                    .settings_mut()
                    .linter
                    .use_workspace_local_config = use_workspace;
                let _ = state.config.try_save_settings();

                let target_path = if use_workspace {
                    ws_path.clone()
                } else {
                    global_json_path.clone()
                };

                /* WHY: If switching and the target file exists, automatically open it */
                if target_path.exists() {
                    *action = Some(AppAction::SelectDocument(target_path));
                }
            }
            ui.add_space(SETTINGS_TOGGLE_SPACING);
        }

        let json_path = if use_workspace {
            workspace_json_path.unwrap_or(global_json_path)
        } else {
            global_json_path
        };

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
                    {
                        if let Some(parent) = json_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if std::fs::write(&json_path, "{\n  \"default\": true\n}\n").is_ok() {
                            *action = Some(AppAction::SelectDocument(json_path));
                        }
                    }
                }
            })
            .show(ui);
    }
}
