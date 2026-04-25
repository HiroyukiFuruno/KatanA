use crate::settings::SETTINGS_TOGGLE_SPACING;
use eframe::egui;

pub(super) struct RuleGroupOps;

impl RuleGroupOps {
    pub(super) fn render_rule_group(
        ui: &mut egui::Ui,
        rule: &dyn katana_markdown_linter::rules::markdown::MarkdownRule,
        config: &mut katana_markdown_linter::rules::markdown::config::MarkdownLintConfig,
        target_path: &std::path::Path,
        search_query: &str,
        force_open: Option<bool>,
    ) {
        if let Some(meta) = rule.official_meta() {
            if meta.properties.is_empty() {
                return;
            }

            let localized_description =
                crate::linter_bridge::MarkdownLinterBridgeOps::rule_description(&meta);
            let search_lower = search_query.to_lowercase();
            let localized_description_lower = localized_description.to_lowercase();
            if !search_lower.is_empty()
                && !meta.code.to_lowercase().contains(&search_lower)
                && !meta.title.to_lowercase().contains(&search_lower)
                && !meta.description.to_lowercase().contains(&search_lower)
                && !localized_description_lower.contains(&search_lower)
            {
                return;
            }

            ui.add_space(SETTINGS_TOGGLE_SPACING);

            /* WHY: Render Rule Header using Accordion to match shortcuts consistency */
            let mut label_job = egui::text::LayoutJob::default();
            label_job.append(
                meta.code,
                0.0,
                egui::TextFormat {
                    font_id: egui::TextStyle::Body.resolve(ui.style()),
                    color: ui.visuals().strong_text_color(),
                    ..Default::default()
                },
            );
            label_job.append(
                &format!("  {localized_description}"),
                0.0,
                egui::TextFormat {
                    font_id: egui::TextStyle::Body.resolve(ui.style()),
                    color: ui.visuals().weak_text_color(),
                    ..Default::default()
                },
            );

            crate::widgets::Accordion::new(
                format!("linter_rule_accordion_{}", meta.code),
                label_job,
                |ui| {
                    /* WHY: Render Properties */
                    for prop in meta.properties {
                        match prop.prop_type {
                            katana_markdown_linter::rules::markdown::RulePropertyType::StringArray => {
                                super::properties::RulePropertiesOps::render_string_array_property(
                                    ui,
                                    &meta,
                                    prop,
                                    config,
                                    target_path,
                                );
                            }
                            _ => {
                                super::properties::RulePropertiesOps::render_singleline_property(
                                    ui,
                                    &meta,
                                    prop,
                                    config,
                                    target_path,
                                );
                            }
                        }
                        ui.add_space(2.0);
                    }
                },
            )
            .default_open(true)
            .force_open(force_open)
            .show(ui);
        }
    }
}
