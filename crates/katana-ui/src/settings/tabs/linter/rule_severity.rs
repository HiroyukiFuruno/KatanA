use super::presets::LinterPresetOps;
use super::severity_toggle;
use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use eframe::egui;
use severity_toggle::SEVERITY_SEGMENT_HEIGHT;

const RULE_DESC_SPACING: f32 = 8.0;
const RULE_ROW_SPACING: f32 = 2.0;

pub(crate) struct LinterRuleSeverityOps;

impl LinterRuleSeverityOps {
    pub(crate) fn render_expand_controls(ui: &mut egui::Ui, force_open: &mut Option<bool>) {
        crate::widgets::AlignCenter::new()
            .left(|ui| {
                let i18n_common = &crate::i18n::I18nOps::get().common;
                if ui.button(&i18n_common.expand_all).clicked() {
                    *force_open = Some(true);
                }
                if ui.button(&i18n_common.collapse_all).clicked() {
                    *force_open = Some(false);
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);
    }

    pub(crate) fn render_rule_severities(
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
                for rule in
                    katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
                {
                    let rule_id = rule.id().to_string();
                    let description = rule
                        .official_meta()
                        .map(|meta| {
                            crate::linter_bridge::MarkdownLinterBridgeOps::rule_description(&meta)
                        })
                        .unwrap_or_default();
                    Self::render_rule_severity_row(
                        ui,
                        state,
                        &rule_id,
                        &description,
                        msgs,
                        action,
                    );
                    ui.add_space(RULE_ROW_SPACING);
                }
            },
        )
        .default_open(true)
        .force_open(force_open)
        .show(ui);
    }

    fn render_rule_severity_row(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        rule_id: &str,
        description: &str,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        let current_severity = state
            .config
            .settings
            .settings()
            .linter
            .rule_severity
            .get(rule_id)
            .copied()
            .unwrap_or_default();

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), SEVERITY_SEGMENT_HEIGHT),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                Self::render_rule_severity_toggle(
                    ui,
                    state,
                    rule_id,
                    current_severity,
                    msgs,
                    action,
                );
                Self::render_rule_label(ui, rule_id, description);
            },
        );
    }

    fn render_rule_severity_toggle(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        rule_id: &str,
        current_severity: katana_platform::settings::RuleSeverity,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        let new_severity = severity_toggle::SeverityToggleOps::render_severity_segment_toggle(
            ui,
            &format!("lsev_{rule_id}"),
            current_severity,
            msgs,
        );
        if new_severity == current_severity {
            return;
        }
        let linter = &mut state.config.settings.settings_mut().linter;
        linter
            .rule_severity
            .insert(rule_id.to_string(), new_severity);
        LinterPresetOps::mark_modified(linter);
        let _ = state.config.try_save_settings();
        *action = Some(AppAction::RefreshDiagnostics);
    }

    fn render_rule_label(ui: &mut egui::Ui, rule_id: &str, description: &str) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(rule_id).truncate());
            if !description.is_empty() {
                ui.add_space(RULE_DESC_SPACING);
                ui.label(egui::RichText::new(description).weak().small());
            }
        });
    }
}
