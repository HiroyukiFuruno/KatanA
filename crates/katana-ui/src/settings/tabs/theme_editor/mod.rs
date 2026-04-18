/* WHY: Refactored theme editor entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use crate::settings::*;

pub mod groups;
pub mod modal;
pub mod operations;

pub struct ThemeEditorOps;

impl ThemeEditorOps {
    pub(crate) fn render_custom_color_editor(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let current_colors = state.config.settings.settings().effective_theme_colors();
        let color_i18n = crate::i18n::I18nOps::get().settings.color.clone();
        let mut changed = false;
        let mut new_colors = current_colors.clone();

        let sections = super::theme_color_data::build_color_sections(&color_i18n);

        let show_vertical_line = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;

        for (section_name, grouped_settings) in sections {
            crate::widgets::Accordion::new(
                section_name.clone(),
                egui::RichText::new(section_name.clone())
                    .strong()
                    .size(SECTION_HEADER_SIZE),
                |ui| {
                    ui.add_space(SUBSECTION_SPACING);
                    for (group_opt, settings_list) in grouped_settings {
                        ui.add_space(SUBSECTION_SPACING);
                        groups::ThemeEditorGroupsOps::render_color_group(
                            ui,
                            group_opt,
                            settings_list,
                            &mut new_colors,
                            &mut changed,
                            show_vertical_line,
                        );
                    }
                },
            )
            .default_open(true)
            .show_vertical_line(show_vertical_line)
            .show(ui);
            ui.add_space(SECTION_SPACING);
        }

        if changed {
            state
                .config
                .settings
                .settings_mut()
                .theme
                .custom_color_overrides = Some(new_colors);
            let _ = state.config.try_save_settings();
        }
        ui.add_space(SUBSECTION_SPACING);
        operations::ThemeEditorOperationsOps::render_save_reset_buttons(ui, state);
        modal::ThemeEditorModalOps::render_save_modal(ui, state);
    }
}
