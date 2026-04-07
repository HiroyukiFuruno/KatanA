use super::types::*;

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
    }
    if is_selected {
        config.settings.settings_mut().theme.active_custom_theme = None;
    }
    let _ = config.try_save_settings();
}
use crate::settings::*;
use crate::theme_bridge;

use katana_platform::theme::{ThemeMode, ThemePreset};

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

        SettingsOps::section_header(ui, &crate::i18n::I18nOps::get().settings.theme.icon_pack);

        let mut current_pack = state.config.settings.settings().theme.icon_pack.clone();

        let available_packs = [
            ("katana-icon", "Katana Core (Default)"),
            ("material-symbols", "Material Symbols"),
            ("lucide", "Lucide"),
            ("tabler-icons", "Tabler Icons"),
            ("heroicons", "Heroicons"),
            ("feather", "Feather"),
        ];

        let selected_name = available_packs
            .iter()
            .find(|(id, _)| *id == current_pack)
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| current_pack.clone());

        egui::ComboBox::from_id_source("icon_pack_combobox")
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                for (id, display_name) in available_packs.iter() {
                    let is_selected = current_pack == *id;
                    let response = ui.add(
                        egui::Button::selectable(is_selected, *display_name)
                            .frame_when_inactive(true),
                    );
                    if response.clicked() {
                        current_pack = id.to_string();
                    }
                }
            });

        if current_pack != state.config.settings.settings().theme.icon_pack {
            state.config.settings.settings_mut().theme.icon_pack = current_pack.clone();
            crate::icon::IconRegistry::install_pack_by_id(ui.ctx(), &current_pack);
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SUBSECTION_SPACING);
        ui.label(
            egui::RichText::new(crate::i18n::I18nOps::get().settings.preview.heading.clone())
                .weak(),
        );

        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(SECTION_SPACING);
                    ui.add(crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Large));
                    ui.add_space(SECTION_SPACING);
                    ui.add(crate::Icon::FolderOpen.ui_image(ui, crate::icon::IconSize::Large));
                    ui.add_space(SECTION_SPACING);
                    ui.add(crate::Icon::Settings.ui_image(ui, crate::icon::IconSize::Large));
                    ui.add_space(SECTION_SPACING);
                    ui.add(crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Large));
                    ui.add_space(SECTION_SPACING);
                    ui.add(crate::Icon::Success.ui_image(ui, crate::icon::IconSize::Large));
                });
            })
            .show(ui);

        ui.add_space(SECTION_SPACING);

        let is_open = state
            .config
            .settings
            .settings()
            .theme
            .custom_color_overrides
            .is_some();
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
            |ui| super::theme_editor::render_custom_color_editor(ui, state),
        )
        .default_open(is_open)
        .show(ui);
    }

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

        let custom_themes = state.config.settings.settings().theme.custom_themes.clone();
        if !custom_themes.is_empty() {
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
                let bg_color = theme_bridge::ThemeBridgeOps::rgb_to_color32(
                    custom_theme.colors.system.background,
                );
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
                                .button(
                                    crate::i18n::I18nOps::get().settings.theme.duplicate.clone(),
                                )
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
                                        crate::Icon::Remove
                                            .ui_image(ui, crate::icon::IconSize::Medium),
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
        for preset in presets {
            let is_selected = state.config.settings.settings().theme.preset == **preset;
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
