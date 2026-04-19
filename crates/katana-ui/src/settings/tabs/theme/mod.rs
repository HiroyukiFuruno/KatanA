/* WHY: Refactored theme settings to manage complex state and layout rules while staying within architectural line limits. */

use super::types::*;
use crate::settings::*;

mod custom;
mod presets;

impl ThemeTabOps {
    pub(crate) fn render_theme_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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
        ui.add_space(SECTION_SPACING);
        Self::render_theme_preset_selector(ui, state);
        ui.add_space(SECTION_SPACING);

        let show_line = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;
        crate::widgets::Accordion::new(
            "custom_color_overrides_accordion",
            egui::RichText::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .theme
                    .custom_colors
                    .clone(),
            )
            .strong()
            .size(SECTION_HEADER_SIZE),
            |ui| super::theme_editor::ThemeEditorOps::render_custom_color_editor(ui, state),
        )
        .default_open(false)
        .show_vertical_line(show_line)
        .show(ui);
    }
}
