/* WHY: Isolated color group rendering to manage UI complexity and maintain strict line limits. */

const ADVANCED_GROUP_BODY_INDENT: f32 = 10.0;

pub struct ThemeEditorGroupsOps;

impl ThemeEditorGroupsOps {
    pub(crate) fn render_color_group(
        ui: &mut egui::Ui,
        group_opt: Option<String>,
        settings_list: Vec<(String, crate::settings::tabs::types::ColorPropType)>,
        new_colors: &mut katana_platform::theme::ThemeColors,
        changed: &mut bool,
        show_vertical_line: bool,
        force_open: Option<bool>,
    ) {
        if let Some(group_name) = group_opt {
            crate::widgets::Accordion::new(group_name.clone(), group_name.clone(), |ui| {
                ui.add_space(crate::settings::SUBSECTION_SPACING);
                for (lbl, prop) in settings_list {
                    *changed |= prop.render_row(ui, new_colors, &lbl);
                    ui.add_space(crate::settings::SUBSECTION_SPACING);
                }
            })
            .default_open(true)
            .force_open(force_open)
            .indent(ADVANCED_GROUP_BODY_INDENT)
            .show_vertical_line(show_vertical_line)
            .show(ui);
        } else {
            for (lbl, prop) in settings_list {
                *changed |= prop.render_row(ui, new_colors, &lbl);
                ui.add_space(crate::settings::SUBSECTION_SPACING);
            }
        }
    }
}
