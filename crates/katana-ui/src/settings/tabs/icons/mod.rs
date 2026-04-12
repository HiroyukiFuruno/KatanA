use super::types::*;
use crate::settings::*;
use crate::widgets::AlignCenter;

pub mod colors;
mod general;
pub mod list;
pub mod popups;
pub mod row;
pub mod table;

impl IconsTabOps {
    /* WHY: Renders the primary entry point for the Icon settings tab. */
    pub(crate) fn render_icons_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let i18n = crate::i18n::I18nOps::get();
        popups::IconsPopupsOps::render(ui, state);

        let mut current_pack = state.config.settings.settings().theme.icon_pack.clone();

        let available_packs = [
            ("katana", "KatanA (Default)"),
            ("material-symbols", "Material Symbols"),
            ("lucide", "Lucide"),
            ("tabler-icons", "Tabler Icons"),
            ("heroicons", "Heroicons"),
            ("feather", "Feather"),
        ];

        let mut icon_settings = state.config.settings.settings().icon.clone();
        let mut settings_changed = false;

        let selected_name = if let Some(preset_name) = &icon_settings.active_preset {
            preset_name.clone()
        } else {
            available_packs
                .iter()
                .find(|(id, _)| *id == current_pack)
                .map(|(_, name)| name.to_string())
                .unwrap_or_else(|| current_pack.clone())
        };

        /* WHY: Horizontal alignment using AlignCenter to satisfy vertical centering rules. */
        AlignCenter::new()
            .content(|ui| {
                egui::ComboBox::from_id_source("icon_pack_combobox")
                    .selected_text(selected_name.clone())
                    .show_ui(ui, |ui| {
                        /* WHY: Base built-in packs. */
                        for (id, display_name) in available_packs.iter() {
                            let is_selected =
                                icon_settings.active_preset.is_none() && current_pack == *id;
                            let response = ui.add(
                                egui::Button::selectable(is_selected, *display_name)
                                    .frame_when_inactive(true),
                            );
                            if response.clicked() {
                                current_pack = id.to_string();
                                icon_settings.active_preset = None;
                                icon_settings.active_overrides.clear();
                                settings_changed = true;
                            }
                        }

                        if !icon_settings.custom_presets.is_empty() {
                            ui.separator();
                            /* WHY: Custom saved presets. */
                            for preset in &icon_settings.custom_presets {
                                let is_selected = icon_settings.active_preset.as_deref()
                                    == Some(preset.name.as_str());
                                let response = ui.add(
                                    egui::Button::selectable(is_selected, &preset.name)
                                        .frame_when_inactive(true),
                                );
                                if response.clicked() {
                                    icon_settings.active_preset = Some(preset.name.clone());
                                    icon_settings.active_overrides = preset.overrides.clone();
                                    settings_changed = true;
                                }
                            }
                        }
                    });
            })
            .show(ui);

        if current_pack != state.config.settings.settings().theme.icon_pack {
            state.config.settings.settings_mut().theme.icon_pack = current_pack.clone();
            crate::icon::IconRegistry::install_pack_by_id(
                ui.ctx(),
                &current_pack,
                &state.config.settings.settings().icon,
            );
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SECTION_SPACING);

        let is_advanced_open = ui
            .data(|d| d.get_temp::<bool>(egui::Id::new("icons_advanced_is_open")))
            .unwrap_or(false);

        if is_advanced_open {
            const BOTTOM_PANEL_MARGIN: f32 = 8.0;
            egui::TopBottomPanel::bottom("icon_settings_actions_panel")
                .frame(egui::Frame::none().inner_margin(egui::vec2(0.0, BOTTOM_PANEL_MARGIN)))
                .show_inside(ui, |ui| {
                    /* WHY: Use AlignCenter instead of ui.horizontal() for bottom actions. */
                    AlignCenter::new()
                        .content(|ui| {
                            /* WHY: Action buttons for preset management fixed at the bottom. */
                            if ui.button(&i18n.settings.icons.save_preset).clicked() {
                                ui.data_mut(|d| {
                                    d.insert_temp::<bool>(
                                        egui::Id::new("katana_icon_saving_preset"),
                                        true,
                                    )
                                });
                            }

                            if !icon_settings.active_overrides.is_empty()
                                && ui.button(&i18n.settings.icons.revert_default).clicked()
                            {
                                icon_settings.active_overrides.clear();
                                icon_settings.active_preset = None;
                                settings_changed = true;
                            }

                            if let Some(active_preset) = &icon_settings.active_preset {
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
                                        &crate::i18n::I18nOps::get().settings.theme.delete_custom,
                                    )
                                    .clicked();
                                if clicked {
                                    let old_name = active_preset.clone();
                                    icon_settings.custom_presets.retain(|p| p.name != old_name);
                                    icon_settings.active_preset = None;
                                    icon_settings.active_overrides.clear();
                                    settings_changed = true;
                                }
                            }
                        })
                        .show(ui);
                });
        }

        list::IconsListOps::render(
            ui,
            state,
            i18n,
            &mut icon_settings,
            &current_pack,
            &mut settings_changed,
        );

        if settings_changed {
            state.config.settings.settings_mut().icon = icon_settings;
            ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_pending_icon_update"), true));
        }

        if ui.data(|d| {
            d.get_temp::<bool>(egui::Id::new("katana_pending_icon_update"))
                .unwrap_or(false)
        }) && !ui.input(|i| i.pointer.any_down())
        {
            crate::icon::IconRegistry::install_pack_by_id(
                ui.ctx(),
                &current_pack,
                &state.config.settings.settings().icon,
            );
            let _ = state.config.try_save_settings();
            ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_pending_icon_update"), false));
        }
    }
}
