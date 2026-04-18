/* WHY: Encapsulated custom theme management logic to maintain modularity and isolation of user-defined theme data. */

use crate::settings::*;
use crate::theme_bridge;

pub struct CustomThemeOps;

impl CustomThemeOps {
    pub(crate) fn render_custom_theme_list(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let custom_themes = state.config.settings.settings().theme.custom_themes.clone();
        if custom_themes.is_empty() {
            return;
        }

        ui.add_space(SECTION_SPACING);
        ui.label(
            egui::RichText::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .theme
                    .custom_section
                    .clone(),
            )
            .weak(),
        );
        for (idx, custom_theme) in custom_themes.iter().enumerate() {
            let is_selected = state
                .config
                .settings
                .settings()
                .theme
                .custom_color_overrides
                .as_ref()
                == Some(&custom_theme.colors);
            let bg_color =
                theme_bridge::ThemeBridgeOps::rgb_to_color32(custom_theme.colors.system.background);
            let accent_color =
                theme_bridge::ThemeBridgeOps::rgb_to_color32(custom_theme.colors.system.accent);
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
                    let custom_fill = if is_selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        crate::theme_bridge::TRANSPARENT
                    };
                    let response = ui.add(
                        egui::Button::selectable(is_selected, &custom_theme.name)
                            .frame_when_inactive(true)
                            .fill(custom_fill),
                    );
                    if response.clicked() && !is_selected {
                        state
                            .config
                            .settings
                            .settings_mut()
                            .theme
                            .custom_color_overrides = Some(custom_theme.colors.clone());
                        state
                            .config
                            .settings
                            .settings_mut()
                            .theme
                            .active_custom_theme = Some(custom_theme.name.clone());
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
                                    format!("{} copy", custom_theme.name),
                                );
                                d.insert_temp(
                                    egui::Id::new("duplicate_theme_colors"),
                                    custom_theme.colors.clone(),
                                );
                            });
                            ui.close();
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon_bg = if ui.visuals().dark_mode {
                            crate::theme_bridge::TRANSPARENT
                        } else {
                            crate::theme_bridge::ThemeBridgeOps::from_gray(
                                crate::shell_ui::LIGHT_MODE_ICON_BG,
                            )
                        };
                        let clicked = ui
                            .add(
                                egui::Button::image(
                                    crate::Icon::Remove.ui_image(ui, crate::icon::IconSize::Medium),
                                )
                                .fill(icon_bg),
                            )
                            .on_hover_text(
                                crate::i18n::I18nOps::get()
                                    .settings
                                    .theme
                                    .delete_custom
                                    .clone(),
                            )
                            .clicked();
                        if clicked {
                            apply_theme_delete(&mut state.config, idx, is_selected);
                        }
                    });
                })
                .show(ui);
        }
    }
}

fn apply_theme_delete(
    config: &mut crate::state::config::ConfigState,
    idx: usize,
    is_selected: bool,
) {
    config
        .settings
        .settings_mut()
        .theme
        .custom_themes
        .remove(idx);
    if is_selected {
        config.settings.settings_mut().theme.custom_color_overrides = None;
        config.settings.settings_mut().theme.active_custom_theme = None;
    }
    let _ = config.try_save_settings();
}
