use eframe::egui;

pub(super) struct PropertiesHelpersOps;

impl PropertiesHelpersOps {
    pub(super) fn save_property(
        ui: &mut egui::Ui,
        meta: &katana_markdown_linter::rules::markdown::OfficialRuleMeta,
        prop: &katana_markdown_linter::rules::markdown::RuleProperty,
        json_val: serde_json::Value,
        config: &mut katana_markdown_linter::rules::markdown::config::MarkdownLintConfig,
        target_path: &std::path::Path,
    ) {
        config.set_rule_property(meta.code, prop.key, json_val);
        let _ = config.save(target_path);
        ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_pending_linter_update"), true));
    }
}
