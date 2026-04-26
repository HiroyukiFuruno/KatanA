/* WHY: Encapsulated custom theme management logic to maintain modularity and isolation of user-defined theme data. */

use super::custom_row::CustomThemeRowOps;
use crate::settings::*;
use crate::theme_bridge;
use katana_platform::settings::CustomTheme;

pub struct CustomThemeOps;

impl CustomThemeOps {
    pub(crate) fn render_custom_theme_list(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let custom_themes = state.config.settings.settings().theme.custom_themes.clone();
        if custom_themes.is_empty() {
            return;
        }

        ui.add_space(SECTION_SPACING);
        ui.label(
            egui::RichText::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .theme
                    .custom_section
                    .clone(),
            )
            .weak(),
        );
        for (idx, custom_theme) in custom_themes.iter().enumerate() {
            let is_selected = state
                .config
                .settings
                .settings()
                .theme
                .active_custom_theme
                .as_deref()
                == Some(custom_theme.name.as_str());
            let bg_color =
                theme_bridge::ThemeBridgeOps::rgb_to_color32(custom_theme.colors.system.background);
            let accent_color =
                theme_bridge::ThemeBridgeOps::rgb_to_color32(custom_theme.colors.system.accent);
            let row_width = ui.available_width();
            let row_height = ui.spacing().interact_size.y;
            ui.allocate_ui_with_layout(
                egui::vec2(row_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = CustomThemeRowOps::ROW_SPACING;
                    CustomThemeRowOps::render_swatch(ui, bg_color, accent_color);
                    let response =
                        CustomThemeRowOps::render_name_button(ui, custom_theme, is_selected);
                    if response.clicked() && !is_selected {
                        Self::apply_theme_selection(state, custom_theme);
                    }
                    Self::render_duplicate_menu(response, custom_theme);
                    Self::render_delete_button(ui, state, idx, is_selected);
                },
            );
        }
    }

    fn apply_theme_selection(state: &mut crate::app_state::AppState, custom_theme: &CustomTheme) {
        state
            .config
            .settings
            .settings_mut()
            .theme
            .custom_color_overrides = Some(custom_theme.colors.clone());
        state
            .config
            .settings
            .settings_mut()
            .theme
            .active_custom_theme = Some(custom_theme.name.clone());
        let theme = &mut state.config.settings.settings_mut().theme;
        theme.preset_state.select_user(custom_theme.name.clone());
        theme
            .preset_state
            .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
        let _ = state.config.try_save_settings();
    }

    fn render_duplicate_menu(response: egui::Response, custom_theme: &CustomTheme) {
        response.context_menu(|ui| {
            if ui
                .button(crate::i18n::I18nOps::get().settings.theme.duplicate.clone())
                .clicked()
            {
                ui.data_mut(|data| {
                    data.insert_temp(egui::Id::new("show_save_theme_modal"), true);
                    data.insert_temp(
                        egui::Id::new("custom_theme_name_input"),
                        format!("{} copy", custom_theme.name),
                    );
                    data.insert_temp(
                        egui::Id::new("duplicate_theme_colors"),
                        custom_theme.colors.clone(),
                    );
                });
                ui.close();
            }
        });
    }

    fn render_delete_button(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        idx: usize,
        is_selected: bool,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clicked = CustomThemeRowOps::render_delete_button(ui).clicked();
            if clicked {
                apply_theme_delete(&mut state.config, idx, is_selected);
            }
        });
    }
}

fn apply_theme_delete(
    config: &mut crate::state::config::ConfigState,
    idx: usize,
    is_selected: bool,
) {
    config
        .settings
        .settings_mut()
        .theme
        .custom_themes
        .remove(idx);
    if is_selected {
        config.settings.settings_mut().theme.custom_color_overrides = None;
        config.settings.settings_mut().theme.active_custom_theme = None;
        let theme = &mut config.settings.settings_mut().theme;
        let preset = theme.preset;
        theme
            .preset_state
            .select_built_in(format!("{preset:?}"), preset.display_name());
    }
    let theme = &mut config.settings.settings_mut().theme;
    theme
        .preset_state
        .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
    let _ = config.try_save_settings();
}
