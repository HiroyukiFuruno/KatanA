/* WHY: Encapsulated theme editor state transitions and persistence logic to ensure predictable data flow. */

use crate::settings::*;

pub struct ThemeEditorOperationsOps;

impl ThemeEditorOperationsOps {
    pub(crate) fn render_save_reset_buttons(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let active_custom = state
            .config
            .settings
            .settings()
            .theme
            .active_custom_theme
            .clone();
        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                let limit_reached = state.config.settings.settings().theme.custom_themes.len()
                    >= katana_platform::settings::MAX_CUSTOM_THEMES;
                ui.add_enabled_ui(!limit_reached, |ui| {
                    if ui
                        .button(
                            crate::i18n::I18nOps::get()
                                .settings
                                .theme
                                .save_custom_theme
                                .clone(),
                        )
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("show_save_theme_modal"), true)
                        });
                        let name_id = egui::Id::new("custom_theme_name_input");
                        if let Some(name) = &active_custom {
                            ui.data_mut(|d| d.insert_temp(name_id, format!("{} copy", name)));
                        } else {
                            ui.data_mut(|d| d.insert_temp(name_id, String::new()));
                        }
                    }
                });
                if state
                    .config
                    .settings
                    .settings()
                    .theme
                    .custom_color_overrides
                    .is_some()
                {
                    ui.add_space(SUBSECTION_SPACING);
                    if ui
                        .button(
                            crate::i18n::I18nOps::get()
                                .settings
                                .theme
                                .reset_custom
                                .clone(),
                        )
                        .clicked()
                    {
                        reset_custom_theme(state, &active_custom);
                        let _ = state.config.try_save_settings();
                    }
                }
            },
        );
    }
}

fn reset_custom_theme(state: &mut crate::app_state::AppState, active_custom: &Option<String>) {
    if let Some(name) = active_custom {
        let found = state
            .config
            .settings
            .settings()
            .theme
            .custom_themes
            .iter()
            .find(|t| t.name == *name)
            .map(|t| t.colors.clone());
        if let Some(colors) = found {
            state
                .config
                .settings
                .settings_mut()
                .theme
                .custom_color_overrides = Some(colors);
            let theme = &mut state.config.settings.settings_mut().theme;
            theme.preset_state.select_user(name);
            theme
                .preset_state
                .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
        } else {
            state
                .config
                .settings
                .settings_mut()
                .theme
                .custom_color_overrides = None;
            state
                .config
                .settings
                .settings_mut()
                .theme
                .active_custom_theme = None;
            select_current_built_in_theme(state);
        }
    } else {
        state
            .config
            .settings
            .settings_mut()
            .theme
            .custom_color_overrides = None;
        state
            .config
            .settings
            .settings_mut()
            .theme
            .active_custom_theme = None;
        select_current_built_in_theme(state);
    }
}

fn select_current_built_in_theme(state: &mut crate::app_state::AppState) {
    let theme = &mut state.config.settings.settings_mut().theme;
    let preset = theme.preset;
    theme
        .preset_state
        .select_built_in(format!("{preset:?}"), preset.display_name());
    theme
        .preset_state
        .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
}
