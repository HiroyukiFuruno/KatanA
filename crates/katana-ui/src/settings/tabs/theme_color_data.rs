use crate::settings::tabs::types::*;

pub(super) type ColorSection = (String, Vec<(Option<String>, Vec<(String, ColorPropType)>)>);

pub(super) fn build_color_sections(
    color_i18n: &crate::i18n::SettingsColorMessages,
) -> Vec<ColorSection> {
    let system_settings = vec![
        (
            Some(color_i18n.group_basic.clone()),
            vec![
                (
                    color_i18n.background.clone(),
                    ColorPropType::Rgb(|c| c.system.background, |c, r| c.system.background = r),
                ),
                (
                    color_i18n.panel_background.clone(),
                    ColorPropType::Rgb(
                        |c| c.system.panel_background,
                        |c, r| c.system.panel_background = r,
                    ),
                ),
            ],
        ),
        (
            Some(color_i18n.group_text.clone()),
            vec![
                (
                    color_i18n.text.clone(),
                    ColorPropType::Rgb(|c| c.system.text, |c, r| c.system.text = r),
                ),
                (
                    color_i18n.text_secondary.clone(),
                    ColorPropType::Rgb(
                        |c| c.system.text_secondary,
                        |c, r| c.system.text_secondary = r,
                    ),
                ),
                (
                    color_i18n.success_text.clone(),
                    ColorPropType::Rgb(|c| c.system.success_text, |c, r| c.system.success_text = r),
                ),
                (
                    color_i18n.warning_text.clone(),
                    ColorPropType::Rgb(|c| c.system.warning_text, |c, r| c.system.warning_text = r),
                ),
                (
                    color_i18n.error_text.clone(),
                    ColorPropType::Rgb(|c| c.system.error_text, |c, r| c.system.error_text = r),
                ),
            ],
        ),
        (
            Some(color_i18n.group_ui_elements.clone()),
            vec![
                (
                    color_i18n.title_bar_text.clone(),
                    ColorPropType::Rgb(
                        |c| c.system.title_bar_text,
                        |c, r| c.system.title_bar_text = r,
                    ),
                ),
                (
                    color_i18n.file_tree_text.clone(),
                    ColorPropType::Rgb(
                        |c| c.system.file_tree_text,
                        |c, r| c.system.file_tree_text = r,
                    ),
                ),
                (
                    color_i18n.accent.clone(),
                    ColorPropType::Rgb(|c| c.system.accent, |c, r| c.system.accent = r),
                ),
                (
                    color_i18n.selection.clone(),
                    ColorPropType::Rgb(|c| c.system.selection, |c, r| c.system.selection = r),
                ),
                (
                    color_i18n.border.clone(),
                    ColorPropType::Rgb(|c| c.system.border, |c, r| c.system.border = r),
                ),
                (
                    color_i18n.button_background.clone(),
                    ColorPropType::Rgba(
                        |c| c.system.button_background,
                        |c, r| c.system.button_background = r,
                    ),
                ),
                (
                    color_i18n.button_active_background.clone(),
                    ColorPropType::Rgba(
                        |c| c.system.button_active_background,
                        |c, r| c.system.button_active_background = r,
                    ),
                ),
                (
                    color_i18n.active_file_highlight.clone(),
                    ColorPropType::Rgba(
                        |c| c.system.active_file_highlight,
                        |c, r| c.system.active_file_highlight = r,
                    ),
                ),
            ],
        ),
    ];
    let code_settings = vec![(
        None,
        vec![
            (
                color_i18n.code_background.clone(),
                ColorPropType::Rgb(|c| c.code.background, |c, r| c.code.background = r),
            ),
            (
                color_i18n.code_text.clone(),
                ColorPropType::Rgb(|c| c.code.text, |c, r| c.code.text = r),
            ),
            (
                color_i18n.highlight.clone(),
                ColorPropType::Rgb(|c| c.code.selection, |c, r| c.code.selection = r),
            ),
            (
                color_i18n.line_number_text.clone(),
                ColorPropType::Rgb(
                    |c| c.code.line_number_text,
                    |c, r| c.code.line_number_text = r,
                ),
            ),
            (
                color_i18n.line_number_active_text.clone(),
                ColorPropType::Rgb(
                    |c| c.code.line_number_active_text,
                    |c, r| c.code.line_number_active_text = r,
                ),
            ),
            (
                color_i18n.current_line_background.clone(),
                ColorPropType::Rgba(
                    |c| c.code.current_line_background,
                    |c, r| c.code.current_line_background = r,
                ),
            ),
            (
                color_i18n.hover_line_background.clone(),
                ColorPropType::Rgba(
                    |c| c.code.hover_line_background,
                    |c, r| c.code.hover_line_background = r,
                ),
            ),
            (
                color_i18n.search_match.clone(),
                ColorPropType::Rgba(|c| c.code.search_match, |c, r| c.code.search_match = r),
            ),
            (
                color_i18n.search_active.clone(),
                ColorPropType::Rgba(|c| c.code.search_active, |c, r| c.code.search_active = r),
            ),
        ],
    )];
    let preview_settings = vec![(
        None,
        vec![
            (
                color_i18n.preview_background.clone(),
                ColorPropType::Rgb(|c| c.preview.background, |c, r| c.preview.background = r),
            ),
            (
                color_i18n.preview_text.clone(),
                ColorPropType::Rgb(|c| c.preview.text, |c, r| c.preview.text = r),
            ),
            (
                color_i18n.warning_text.clone(),
                ColorPropType::Rgb(
                    |c| c.preview.warning_text,
                    |c, r| c.preview.warning_text = r,
                ),
            ),
            (
                color_i18n.highlight.clone(),
                ColorPropType::Rgb(|c| c.preview.selection, |c, r| c.preview.selection = r),
            ),
            (
                color_i18n.border.clone(),
                ColorPropType::Rgb(|c| c.preview.border, |c, r| c.preview.border = r),
            ),
            (
                color_i18n.hover_line_background.clone(),
                ColorPropType::Rgba(
                    |c| c.preview.hover_line_background,
                    |c, r| c.preview.hover_line_background = r,
                ),
            ),
        ],
    )];
    vec![
        (color_i18n.section_system.clone(), system_settings),
        (color_i18n.section_code.clone(), code_settings),
        (color_i18n.section_preview.clone(), preview_settings),
    ]
}
