/* WHY: Isolated theme saving modal logic to maintain UI focus and satisfy architectural line limits. */

use crate::settings::*;

const THEME_SAVE_MODAL_WIDTH: f32 = 400.0;
const THEME_SAVE_INPUT_WIDTH: f32 = 260.0;

pub struct ThemeEditorModalOps;

impl ThemeEditorModalOps {
    pub(crate) fn render_save_modal(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let modal_id = egui::Id::new("show_save_theme_modal");
        if !ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false)) {
            return;
        }
        let mut close = false;
        let i18n = crate::i18n::I18nOps::get();
        egui::Window::new(i18n.settings.theme.save_custom_theme_title.clone())
            .collapsible(false)
            .resizable(false)
            .min_width(THEME_SAVE_MODAL_WIDTH)
            .max_width(THEME_SAVE_MODAL_WIDTH)
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
                        let re = ui.add(
                            egui::TextEdit::singleline(&mut name)
                                .desired_width(THEME_SAVE_INPUT_WIDTH),
                        );
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
    }
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
    s.theme.preset_state.select_user(name);
    s.theme
        .preset_state
        .sync_user_preset_names(s.theme.custom_themes.iter().map(|preset| &preset.name));
    let _ = state.config.try_save_settings();
}
