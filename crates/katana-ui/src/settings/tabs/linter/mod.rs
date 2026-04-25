use super::types::*;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::*;
mod advanced;
mod properties;
mod properties_helpers;
mod rule_group;
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

        let mut is_advanced_open = ui
            .data(|d| d.get_temp::<bool>(egui::Id::new("linter_advanced_is_open")))
            .unwrap_or(false);

        if is_advanced_open {
            advanced::LinterAdvancedSettingsOps::render_advanced_settings(
                ui,
                state,
                linter_msgs,
                &mut is_advanced_open,
            );
        } else {
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

            let mut force_open: Option<bool> = None;
            if enabled {
                crate::widgets::AlignCenter::new()
                    .left(|ui| {
                        let i18n_common = &crate::i18n::I18nOps::get().common;
                        if ui.button(&i18n_common.expand_all).clicked() {
                            force_open = Some(true);
                        }
                        if ui.button(&i18n_common.collapse_all).clicked() {
                            force_open = Some(false);
                        }
                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                    })
                    .show(ui);

                ui.add_space(SUBSECTION_SPACING);

                Self::render_rule_severities(ui, state, linter_msgs, &mut action, force_open);

                ui.add_space(SUBSECTION_SPACING);

                WorkspaceSettingsOps::render_workspace_settings(
                    ui,
                    state,
                    linter_msgs,
                    &mut is_advanced_open,
                    force_open,
                );
            }
        }

        ui.data_mut(|d| d.insert_temp(egui::Id::new("linter_advanced_is_open"), is_advanced_open));

        if ui.data(|d| {
            d.get_temp::<bool>(egui::Id::new("katana_pending_linter_update"))
                .unwrap_or(false)
        }) {
            ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_pending_linter_update"), false));
            action = Some(AppAction::RefreshDiagnostics);
        }

        action
    }

    fn render_rule_severities(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
        force_open: Option<bool>,
    ) {
        crate::widgets::Accordion::new(
            "linter_rule_severities_accordion",
            egui::RichText::new(&msgs.rule_severities).strong(),
            |ui| {
                /* WHY: get_user_configurable_rules() filters out internal-only rules (e.g. BrokenLinkRule)
                 * and returns them sorted by ID for consistent presentation. */
                for rule in
                    katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
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

                                if let Some(meta) = rule.official_meta() {
                                    let description =
                                        crate::linter_bridge::MarkdownLinterBridgeOps::rule_description(
                                            &meta,
                                        );
                                    if !description.is_empty() {
                                        ui.add_space(RULE_DESC_SPACING);
                                        ui.label(
                                            egui::RichText::new(description).weak().small(),
                                        );
                                    }
                                }
                            });
                        },
                    );

                    ui.add_space(2.0);
                }
            },
        )
        .default_open(true)
        .force_open(force_open)
        .show(ui);
    }
}
