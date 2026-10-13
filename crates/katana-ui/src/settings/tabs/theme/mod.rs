/* WHY: Refactored theme settings to manage complex state and layout rules while staying within architectural line limits. */

use super::types::*;
use crate::settings::*;

mod custom;
mod custom_row;
mod preset_controls;
mod presets;

const THEME_ADVANCED_SEARCH_FILTER: &str = "theme_advanced_search_filter";

impl ThemeTabOps {
    pub(crate) fn render_theme_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let mut is_advanced_open = ui
            .data(|data| data.get_temp::<bool>(egui::Id::new("theme_advanced_is_open")))
            .unwrap_or(false);

        if is_advanced_open {
            Self::render_advanced_settings(ui, state, &mut is_advanced_open);
        } else {
            Self::render_normal_theme_tab(ui, state, &mut is_advanced_open);
        }

        ui.data_mut(|data| {
            data.insert_temp(egui::Id::new("theme_advanced_is_open"), is_advanced_open)
        });
    }

    fn render_normal_theme_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        is_advanced_open: &mut bool,
    ) {
        Self::render_theme_preset_selector(ui, state, is_advanced_open);

        ui.add_space(SECTION_SPACING);
        SettingsOps::section_header(
            ui,
            crate::i18n::I18nOps::get()
                .settings
                .theme
                .ui_contrast_offset
                .as_str(),
        );
        let mut offset = state.config.settings.settings().theme.ui_contrast_offset;
        let original_offset = offset;
        let slider = egui::Slider::new(&mut offset, -100.0..=100.0)
            .step_by(1.0)
            .suffix(" %");
        if SettingsOps::add_styled_slider(ui, slider).changed() {
            state
                .config
                .settings
                .settings_mut()
                .theme
                .ui_contrast_offset = offset;
            if offset != original_offset {
                let colors = state.config.settings.settings().effective_theme_colors();
                ui.ctx()
                    .set_visuals(crate::theme_bridge::ThemeBridgeOps::visuals_from_theme(
                        &colors,
                    ));
            }
        }

        super::theme_editor::modal::ThemeEditorModalOps::render_save_modal(ui, state);
    }

    fn render_advanced_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        is_advanced_open: &mut bool,
    ) {
        let mut force_open: Option<bool> = None;
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
        ui.add_space(crate::settings::SETTINGS_TOGGLE_SPACING);
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

        ui.add_space(crate::settings::SETTINGS_TOGGLE_SPACING);
        let mut search_query = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new(THEME_ADVANCED_SEARCH_FILTER))
                .unwrap_or_default()
        });
        let i18n = crate::i18n::I18nOps::get();
        let search_response = crate::widgets::SearchBar::simple(&mut search_query)
            .hint_text(&i18n.settings.theme.search_placeholder)
            .show_search_icon(true)
            .id_source(THEME_ADVANCED_SEARCH_FILTER)
            .show(ui);
        if search_response.changed() {
            let query = search_query.clone();
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new(THEME_ADVANCED_SEARCH_FILTER), query);
            });
        }

        ui.add_space(crate::settings::SETTINGS_TOGGLE_SPACING);
        egui::ScrollArea::vertical()
            .id_salt("theme_advanced_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                super::theme_editor::ThemeEditorOps::render_custom_color_editor(
                    ui,
                    state,
                    &search_query,
                    force_open,
                );
            });
    }
}

#[cfg(test)]
mod mod_tests;
