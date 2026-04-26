use eframe::egui;
use katana_platform::settings::PresetReference;
use katana_platform::settings::PresetSource;
use katana_platform::settings::ThemeSettings;
use katana_platform::theme::ThemePreset;

pub(crate) struct ThemePresetControlsOps;

impl ThemePresetControlsOps {
    pub(crate) fn render(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let theme_settings = state.config.settings.settings().theme.clone();
        let built_in_presets = Self::built_in_presets();
        let user_presets = Self::user_presets(&theme_settings);
        let mut preset_state = theme_settings.preset_state.clone();
        Self::ensure_current_state(&mut preset_state, &theme_settings);

        let i18n = crate::i18n::I18nOps::get();
        let labels = crate::settings::tabs::preset_widget::PresetWidgetLabels {
            title: &i18n.settings.theme.preset,
            save: &i18n.settings.theme.save_custom_theme,
            revert: &i18n.settings.theme.reset_custom,
            advanced: Some(&i18n.common.advanced_settings),
        };
        let response = crate::settings::tabs::preset_widget::PresetWidgetOps::render(
            ui,
            "theme_preset_widget",
            &preset_state,
            &built_in_presets,
            &user_presets,
            labels,
        );
        Self::handle_response(ui, state, response);
    }

    fn handle_response(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        response: crate::settings::tabs::preset_widget::PresetWidgetResponse,
    ) {
        let mut changed = false;
        if let Some(reference) = response.selected {
            changed = Self::apply_reference(state, &reference);
        }
        if response.revert_clicked {
            let base = state
                .config
                .settings
                .settings()
                .theme
                .preset_state
                .base
                .clone();
            if let Some(reference) = base {
                changed = Self::apply_reference(state, &reference);
            }
        }
        if response.save_clicked {
            ui.data_mut(|data| data.insert_temp(egui::Id::new("show_save_theme_modal"), true));
        }
        if response.advanced_clicked {
            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("theme_custom_colors_force_open"), true);
            });
        }
        if changed {
            let colors = state.config.settings.settings().effective_theme_colors();
            ui.ctx()
                .set_visuals(crate::theme_bridge::ThemeBridgeOps::visuals_from_theme(
                    &colors,
                ));
            let _ = state.config.try_save_settings();
        }
    }

    fn apply_reference(
        state: &mut crate::app_state::AppState,
        reference: &PresetReference,
    ) -> bool {
        match reference.source {
            PresetSource::BuiltIn => Self::apply_built_in(state, &reference.id),
            PresetSource::User | PresetSource::Custom => Self::apply_user(state, &reference.id),
            PresetSource::Unknown => false,
        }
    }

    fn apply_built_in(state: &mut crate::app_state::AppState, id: &str) -> bool {
        let Some(preset) = Self::theme_preset_by_id(id) else {
            return false;
        };
        let theme = &mut state.config.settings.settings_mut().theme;
        theme.preset = preset;
        theme.theme = preset.colors().mode.to_theme_string();
        theme.custom_color_overrides = None;
        theme.active_custom_theme = None;
        theme
            .preset_state
            .select_built_in(format!("{preset:?}"), preset.display_name());
        Self::sync_user_presets(theme);
        true
    }

    fn apply_user(state: &mut crate::app_state::AppState, name: &str) -> bool {
        let Some(custom_theme) = state
            .config
            .settings
            .settings()
            .theme
            .custom_themes
            .iter()
            .find(|theme| theme.name == name)
            .cloned()
        else {
            return false;
        };
        let theme = &mut state.config.settings.settings_mut().theme;
        theme.custom_color_overrides = Some(custom_theme.colors);
        theme.active_custom_theme = Some(custom_theme.name.clone());
        theme.preset_state.select_user(custom_theme.name);
        Self::sync_user_presets(theme);
        true
    }

    fn ensure_current_state(
        preset_state: &mut katana_platform::settings::PresetState,
        theme_settings: &ThemeSettings,
    ) {
        if preset_state.current.is_some() {
            return;
        }
        preset_state.select_built_in(
            format!("{:?}", theme_settings.preset),
            theme_settings.preset.display_name(),
        );
    }

    fn sync_user_presets(theme: &mut ThemeSettings) {
        theme
            .preset_state
            .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
    }

    fn built_in_presets() -> Vec<PresetReference> {
        ThemePreset::builtins()
            .into_iter()
            .map(|preset| PresetReference::built_in(format!("{preset:?}"), preset.display_name()))
            .collect()
    }

    fn user_presets(theme: &ThemeSettings) -> Vec<PresetReference> {
        theme
            .custom_themes
            .iter()
            .map(|theme| PresetReference::user(&theme.name))
            .collect()
    }

    fn theme_preset_by_id(id: &str) -> Option<ThemePreset> {
        ThemePreset::builtins()
            .into_iter()
            .find(|preset| format!("{preset:?}") == id)
    }
}
