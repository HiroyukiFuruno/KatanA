use crate::settings::*;

pub(super) fn render_custom_color_editor(
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
                    render_color_group(
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
    render_save_reset_buttons(ui, state);
    render_save_modal(ui, state);
}

fn render_color_group(
    ui: &mut egui::Ui,
    group_opt: Option<String>,
    settings_list: Vec<(String, super::types::ColorPropType)>,
    new_colors: &mut katana_platform::theme::ThemeColors,
    changed: &mut bool,
    show_vertical_line: bool,
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
        .show_vertical_line(show_vertical_line)
        .show(ui);
    } else {
        for (lbl, prop) in settings_list {
            *changed |= prop.render_row(ui, new_colors, &lbl);
            ui.add_space(crate::settings::SUBSECTION_SPACING);
        }
    }
}

fn render_save_reset_buttons(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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
                    ui.data_mut(|d| d.insert_temp(egui::Id::new("show_save_theme_modal"), true));
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
    }
}

fn render_save_modal(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let modal_id = egui::Id::new("show_save_theme_modal");
    if !ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false)) {
        return;
    }
    let mut close = false;
    let i18n = crate::i18n::I18nOps::get();
    egui::Window::new(i18n.settings.theme.save_custom_theme_title.clone())
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
            let name_id = egui::Id::new("custom_theme_name_input");
            let mut name = ui.data(|d| d.get_temp::<String>(name_id).unwrap_or_default());
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.label(
                        crate::i18n::I18nOps::get()
                            .settings
                            .theme
                            .theme_name_label
                            .clone(),
                    );
                    let re = ui.text_edit_singleline(&mut name);
                    re.request_focus();
                    if re.changed() {
                        ui.data_mut(|d| d.insert_temp(name_id, name.clone()));
                    }
                })
                .show(ui);
            ui.add_space(SUBSECTION_SPACING);
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    if ui
                        .button(crate::i18n::I18nOps::get().action.cancel.clone())
                        .clicked()
                    {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(crate::i18n::I18nOps::get().action.save.clone())
                            .clicked()
                            && !name.is_empty()
                        {
                            save_custom_theme(ui, state, &name);
                            close = true;
                        }
                    });
                })
                .show(ui);
            if close || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                    d.insert_temp(modal_id, false);
                    d.remove::<String>(egui::Id::new("custom_theme_name_input"));
                    d.remove::<katana_platform::theme::ThemeColors>(egui::Id::new(
                        "duplicate_theme_colors",
                    ));
                });
            }
        });
    let _ = i18n;
}

fn save_custom_theme(ui: &mut egui::Ui, state: &mut crate::app_state::AppState, name: &str) {
    let dup_id = egui::Id::new("duplicate_theme_colors");
    let mut tc = ui
        .data(|d| d.get_temp::<katana_platform::theme::ThemeColors>(dup_id))
        .unwrap_or_else(|| state.config.settings.settings().effective_theme_colors());
    tc.name = name.to_string();
    let mut themes = state.config.settings.settings().theme.custom_themes.clone();
    if let Some(e) = themes.iter_mut().find(|t| t.name == name) {
        e.colors = tc.clone();
    } else {
        themes.push(katana_platform::settings::CustomTheme {
            name: name.to_string(),
            colors: tc.clone(),
        });
    }
    let s = state.config.settings.settings_mut();
    s.theme.custom_themes = themes;
    s.theme.custom_color_overrides = Some(tc);
    s.theme.active_custom_theme = Some(name.to_string());
    let _ = state.config.try_save_settings();
}
