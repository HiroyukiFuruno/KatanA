/* WHY: Isolated theme preset selection logic to manage modularity and maintain strict line limits for UI components. */

use super::super::types::*;
use crate::settings::*;
use crate::theme_bridge;
use katana_platform::theme::{ThemeMode, ThemePreset};

impl ThemeTabOps {
    pub(crate) fn render_theme_preset_selector(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        SettingsOps::section_header(ui, &crate::i18n::I18nOps::get().settings.theme.preset);
        let show_more_id = ui.id().with("show_more_themes");
        let mut show_more = ui.data_mut(|d| d.get_temp::<bool>(show_more_id).unwrap_or(false));
        const VISIBLE_PRESET_COUNT: usize = 5;

        ui.label(
            egui::RichText::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .theme
                    .dark_section
                    .clone(),
            )
            .weak(),
        );
        let all_presets = ThemePreset::builtins();
        let mut dark_presets: Vec<&ThemePreset> = all_presets
            .iter()
            .filter(|it| it.colors().mode == ThemeMode::Dark)
            .collect();
        if !show_more {
            dark_presets.truncate(VISIBLE_PRESET_COUNT);
        }
        Self::render_preset_group(ui, state, &dark_presets);
        ui.add_space(SECTION_SPACING);

        ui.label(
            egui::RichText::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .theme
                    .light_section
                    .clone(),
            )
            .weak(),
        );
        let mut light_presets: Vec<&ThemePreset> = all_presets
            .iter()
            .filter(|it| it.colors().mode == ThemeMode::Light)
            .collect();
        if !show_more {
            light_presets.truncate(VISIBLE_PRESET_COUNT);
        }
        Self::render_preset_group(ui, state, &light_presets);

        super::custom::CustomThemeOps::render_custom_theme_list(ui, state);

        ui.add_space(SUBSECTION_SPACING);
        let msgs = &crate::i18n::I18nOps::get().settings.theme;
        let toggle_text = if show_more {
            &msgs.show_less
        } else {
            &msgs.show_more
        };
        if ui.link(toggle_text).clicked() {
            show_more = !show_more;
            ui.data_mut(|d| d.insert_temp(show_more_id, show_more));
        }
    }

    pub(crate) fn render_preset_group(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        presets: &[&ThemePreset],
    ) {
        let theme_settings = &state.config.settings.settings().theme;
        let has_custom_theme_selection = theme_settings.active_custom_theme.is_some()
            || theme_settings.custom_color_overrides.is_some();
        for preset in presets {
            let is_selected = !has_custom_theme_selection
                && state.config.settings.settings().theme.preset == **preset;
            let colors = preset.colors();
            let bg_color = theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.background);
            let accent_color = theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.accent);
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(PRESET_SWATCH_SIZE, PRESET_SWATCH_SIZE),
                        egui::Sense::hover(),
                    );
                    let corner = PRESET_SWATCH_SIZE / SWATCH_CORNER_DIVISOR;
                    ui.painter().rect_filled(rect, corner, bg_color);
                    ui.painter()
                        .circle_filled(rect.center(), corner, accent_color);
                    let preset_fill = if is_selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        crate::theme_bridge::TRANSPARENT
                    };
                    let response = ui.add(
                        egui::Button::selectable(is_selected, preset.display_name())
                            .frame_when_inactive(true)
                            .fill(preset_fill),
                    );
                    if response.clicked() && !is_selected {
                        state.config.settings.settings_mut().theme.preset = **preset;
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
                        let _ = state.config.try_save_settings();
                    }
                    response.context_menu(|ui| {
                        if ui
                            .button(crate::i18n::I18nOps::get().settings.theme.duplicate.clone())
                            .clicked()
                        {
                            ui.data_mut(|d| {
                                d.insert_temp(egui::Id::new("show_save_theme_modal"), true);
                                d.insert_temp(
                                    egui::Id::new("custom_theme_name_input"),
                                    format!("{} copy", preset.display_name()),
                                );
                                d.insert_temp(
                                    egui::Id::new("duplicate_theme_colors"),
                                    preset.colors().clone(),
                                );
                            });
                            ui.close();
                        }
                    });
                })
                .show(ui);
        }
    }
}
