use super::*;
use crate::app_state::SettingsTab;
use accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;

fn build_theme_state() -> crate::app_state::AppState {
    let mut state = crate::app_state::AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.config.active_settings_tab = SettingsTab::Theme;
    state
}

#[test]
fn advanced_settings_renders_custom_color_editor_as_separate_view() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1024.0, 768.0))
        .build_ui(|ui| {
            let mut state = build_theme_state();

            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("theme_advanced_is_open"), true);
            });
            ThemeTabOps::render_theme_tab(ui, &mut state);
        });

    harness.run_steps(1);

    let i18n = crate::i18n::I18nOps::get();
    harness.get_by_label(&i18n.common.advanced_settings);
    harness.get_by_label(&i18n.common.expand_all);
    harness.get_by_label(&i18n.common.collapse_all);
    harness.get_by_role(Role::TextInput);
    harness.get_by_label(&i18n.settings.color.section_system);
}

#[test]
fn advanced_settings_filters_color_sections_by_search_query() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1024.0, 768.0))
        .build_ui(|ui| {
            let mut state = build_theme_state();

            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("theme_advanced_is_open"), true);
                data.insert_temp(
                    egui::Id::new(THEME_ADVANCED_SEARCH_FILTER),
                    "code".to_string(),
                );
            });
            ThemeTabOps::render_theme_tab(ui, &mut state);
        });

    harness.run_steps(1);

    let i18n = crate::i18n::I18nOps::get();
    assert!(
        harness
            .query_by_label(&i18n.settings.color.section_system)
            .is_none()
    );
    harness.get_by_label(&i18n.settings.color.section_code);
}

#[test]
fn normal_settings_renders_save_theme_modal_when_requested() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1024.0, 768.0))
        .build_ui(|ui| {
            let mut state = build_theme_state();

            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("show_save_theme_modal"), true);
            });
            ThemeTabOps::render_theme_tab(ui, &mut state);
        });

    harness.run_steps(1);

    let i18n = crate::i18n::I18nOps::get();
    harness.get_by_label(&i18n.settings.theme.save_custom_theme_title);
    harness.get_by_label(&i18n.settings.theme.theme_name_label);
}
