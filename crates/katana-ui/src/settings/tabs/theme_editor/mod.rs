/* WHY: Refactored theme editor entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use crate::settings::*;

pub mod groups;
pub mod modal;
pub mod operations;

const ADVANCED_ACCORDION_BODY_INDENT: f32 = 10.0;

pub struct ThemeEditorOps;

impl ThemeEditorOps {
    pub(crate) fn render_custom_color_editor(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        search_query: &str,
        force_open: Option<bool>,
    ) {
        let current_colors = state.config.settings.settings().effective_theme_colors();
        let color_i18n = crate::i18n::I18nOps::get().settings.color.clone();
        let mut changed = false;
        let mut new_colors = current_colors.clone();

        let sections = super::theme_color_data::build_color_sections(&color_i18n);
        let search_lower = search_query.to_lowercase();

        let show_vertical_line = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;

        for (section_name, grouped_settings) in sections {
            let section_matches = label_matches(&section_name, &search_lower);
            let visible_groups =
                filter_grouped_settings(grouped_settings, &search_lower, section_matches);
            if visible_groups.is_empty() {
                continue;
            }

            crate::widgets::Accordion::new(
                section_name.clone(),
                egui::RichText::new(section_name.clone())
                    .strong()
                    .size(SECTION_HEADER_SIZE),
                |ui| {
                    ui.add_space(SUBSECTION_SPACING);
                    for (group_opt, settings_list) in visible_groups {
                        ui.add_space(SUBSECTION_SPACING);
                        groups::ThemeEditorGroupsOps::render_color_group(
                            ui,
                            group_opt,
                            settings_list,
                            &mut new_colors,
                            &mut changed,
                            show_vertical_line,
                            force_open,
                        );
                    }
                },
            )
            .default_open(true)
            .force_open(force_open)
            .indent(ADVANCED_ACCORDION_BODY_INDENT)
            .show_vertical_line(show_vertical_line)
            .show(ui);
            ui.add_space(SECTION_SPACING);
        }

        if changed {
            let theme = &mut state.config.settings.settings_mut().theme;
            theme.custom_color_overrides = Some(new_colors);
            theme.preset_state.mark_modified();
            theme
                .preset_state
                .sync_user_preset_names(theme.custom_themes.iter().map(|preset| &preset.name));
            let _ = state.config.try_save_settings();
        }
        ui.add_space(SUBSECTION_SPACING);
        operations::ThemeEditorOperationsOps::render_save_reset_buttons(ui, state);
        modal::ThemeEditorModalOps::render_save_modal(ui, state);
    }
}

type ThemeEditorColorSetting = (String, crate::settings::tabs::types::ColorPropType);
type ThemeEditorColorGroup = (Option<String>, Vec<ThemeEditorColorSetting>);

fn filter_grouped_settings(
    grouped_settings: Vec<ThemeEditorColorGroup>,
    search_lower: &str,
    section_matches: bool,
) -> Vec<ThemeEditorColorGroup> {
    if search_lower.is_empty() || section_matches {
        return grouped_settings;
    }

    grouped_settings
        .into_iter()
        .filter_map(|(group_name, settings_list)| {
            let group_matches = group_name
                .as_ref()
                .is_some_and(|name| label_matches(name, search_lower));
            let visible_settings = filter_settings_list(settings_list, search_lower, group_matches);
            if visible_settings.is_empty() {
                None
            } else {
                Some((group_name, visible_settings))
            }
        })
        .collect()
}

fn filter_settings_list(
    settings_list: Vec<(String, crate::settings::tabs::types::ColorPropType)>,
    search_lower: &str,
    group_matches: bool,
) -> Vec<(String, crate::settings::tabs::types::ColorPropType)> {
    if group_matches {
        return settings_list;
    }

    settings_list
        .into_iter()
        .filter(|(label, _)| label_matches(label, search_lower))
        .collect()
}

fn label_matches(label: &str, search_lower: &str) -> bool {
    search_lower.is_empty() || label.to_lowercase().contains(search_lower)
}
