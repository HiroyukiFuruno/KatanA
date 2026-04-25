use super::rule_group::RuleGroupOps;
use crate::i18n::LinterTranslations;
use crate::settings::SETTINGS_TOGGLE_SPACING;
use eframe::egui;

pub(crate) struct LinterAdvancedSettingsOps;

impl LinterAdvancedSettingsOps {
    pub(crate) fn render_advanced_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        is_advanced_open: &mut bool,
    ) {
        crate::widgets::AlignCenter::new()
            .left(|ui| ui.heading(&crate::i18n::I18nOps::get().common.advanced_settings))
            .right(|ui| {
                if ui
                    .button(&crate::i18n::I18nOps::get().common.close)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    *is_advanced_open = false;
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);

        ui.separator();
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut search_query = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("linter_advanced_search_filter"))
                .unwrap_or_default()
        });

        let mut force_open: Option<bool> = None;
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

        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let search_response = crate::widgets::SearchBar::simple(&mut search_query)
            .hint_text(&msgs.search_placeholder)
            .show_search_icon(true)
            .id_source("linter_advanced_search_filter")
            .show(ui);

        if search_response.changed() {
            let q = search_query.clone();
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("linter_advanced_search_filter"), q);
            });
        }

        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let target_path =
            crate::linter_config_bridge::MarkdownLinterConfigOps::target_config_path(state);

        /* WHY: Load the current configuration to populate the UI and save updates */
        let mut config =
            katana_markdown_linter::MarkdownLintConfig::load(&target_path).unwrap_or_default();

        for rule in
            katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
        {
            RuleGroupOps::render_rule_group(
                ui,
                rule.as_ref(),
                &mut config,
                &target_path,
                &search_query,
                force_open,
            );
        }
    }
}
