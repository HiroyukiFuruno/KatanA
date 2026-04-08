use super::types::*;
use crate::settings::*;

impl IconsTabOps {
    pub(crate) fn render_icons_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        Self::render_popups(ui, state);

        let mut current_pack = state.config.settings.settings().theme.icon_pack.clone();

        let available_packs = [
            ("katana", "KatanA (Default)"),
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
            crate::icon::IconRegistry::install_pack_by_id(
                ui.ctx(),
                &current_pack,
                &state.config.settings.settings().icon,
            );
            let _ = state.config.try_save_settings();
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Preset:");
            let mut icon_settings = state.config.settings.settings().icon.clone();
            let mut changed = false;

            let active_preset_name = icon_settings
                .active_preset
                .clone()
                .unwrap_or_else(|| "Custom".to_string());

            egui::ComboBox::from_id_source("icon_preset_combobox")
                .selected_text(active_preset_name)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(icon_settings.active_preset.is_none(), "Custom")
                        .clicked()
                    {
                        icon_settings.active_preset = None;
                        changed = true;
                    }
                    for preset in &icon_settings.custom_presets {
                        let is_selected =
                            icon_settings.active_preset.as_ref() == Some(&preset.name);
                        if ui.selectable_label(is_selected, &preset.name).clicked() {
                            icon_settings.active_preset = Some(preset.name.clone());
                            icon_settings.active_overrides = preset.overrides.clone();
                            changed = true;
                        }
                    }
                });

            if ui.button("Save Preset As...").clicked() {
                ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_icon_saving_preset"), true));
            }
            if ui.button("Revert to Default").clicked() {
                icon_settings.active_overrides.clear();
                icon_settings.active_preset = None;
                changed = true;
            }

            if changed {
                state.config.settings.settings_mut().icon = icon_settings.clone();
                crate::icon::IconRegistry::install_pack_by_id(
                    ui.ctx(),
                    &current_pack,
                    &icon_settings,
                );
                let _ = state.config.try_save_settings();
            }
        });

        ui.add_space(SECTION_SPACING);

        egui::ScrollArea::vertical()
            .id_salt("icon_pack_preview_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                let mut grouped_icons: std::collections::BTreeMap<String, Vec<&crate::icon::Icon>> =
                    std::collections::BTreeMap::new();

                for icon in crate::icon::ALL_ICONS {
                    let name = icon.name();
                    let vendor = if let Some(slash_idx) = name.find('/') {
                        name[..slash_idx].to_string()
                    } else {
                        "katana".to_string()
                    };
                    grouped_icons.entry(vendor).or_default().push(icon);
                }

                for (vendor, icons) in grouped_icons {
                    egui::CollapsingHeader::new(&vendor)
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                const ITEM_SPACING: f32 = 16.0;
                                ui.spacing_mut().item_spacing =
                                    egui::vec2(ITEM_SPACING, ITEM_SPACING);
                                for icon in icons {
                                    let image = icon.image(crate::icon::IconSize::Large);
                                    let mut color = ui.visuals().text_color();

                                    // Use active override color if present
                                    let active_ovs =
                                        crate::icon::IconRegistry::get_active_overrides(ui.ctx());
                                    if let Some(ov) =
                                        active_ovs.as_ref().and_then(|o| o.0.get(icon.name()))
                                    {
                                        if let Some(hex) = &ov.color_hex {
                                            if let Ok(c) = egui::Color32::from_hex(hex) {
                                                color = c;
                                            }
                                        } else if let Some(vc) =
                                            icon.vendor_default_color(ui.visuals().dark_mode)
                                        {
                                            color = vc;
                                        }
                                    } else if let Some(vc) =
                                        icon.vendor_default_color(ui.visuals().dark_mode)
                                    {
                                        color = vc;
                                    }

                                    let mut response =
                                        ui.add(egui::Button::image(image.tint(color)).frame(false));
                                    response = response
                                        .on_hover_text(format!("{} (Click to Edit)", icon.name()));

                                    if response.clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new("katana_editing_icon_id"),
                                                icon.name().to_string(),
                                            )
                                        });
                                    }
                                }
                            });
                        });
                }
            });
    }

    fn render_popups(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let show_save_dialog = ui.data(|d| {
            d.get_temp::<bool>(egui::Id::new("katana_icon_saving_preset"))
                .unwrap_or(false)
        });
        if show_save_dialog {
            let mut preset_name = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new("katana_preset_name_input"))
                    .unwrap_or_default()
            });
            let mut close_dialog = false;

            egui::Window::new("Save Preset As")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Preset Name:");
                    let response = ui.text_edit_singleline(&mut preset_name);
                    response.request_focus();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let mut icon_settings = state.config.settings.settings().icon.clone();
                            if let Some(existing) = icon_settings
                                .custom_presets
                                .iter_mut()
                                .find(|p| p.name == preset_name)
                            {
                                existing.overrides = icon_settings.active_overrides.clone();
                            } else {
                                icon_settings.custom_presets.push(
                                    katana_platform::settings::types::icon::IconPreset {
                                        name: preset_name.clone(),
                                        overrides: icon_settings.active_overrides.clone(),
                                    },
                                );
                            }
                            icon_settings.active_preset = Some(preset_name.clone());
                            state.config.settings.settings_mut().icon = icon_settings;
                            let _ = state.config.try_save_settings();

                            close_dialog = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            if close_dialog {
                ui.data_mut(|d| d.insert_temp(egui::Id::new("katana_icon_saving_preset"), false));
                ui.data_mut(|d| {
                    d.insert_temp(egui::Id::new("katana_preset_name_input"), String::new())
                });
            } else {
                ui.data_mut(|d| {
                    d.insert_temp(egui::Id::new("katana_preset_name_input"), preset_name)
                });
            }
        }

        let editing_icon = ui.data(|d| {
            d.get_temp::<String>(egui::Id::new("katana_editing_icon_id"))
                .unwrap_or_default()
        });
        if !editing_icon.is_empty() {
            let mut close_dialog = false;
            let mut icon_settings = state.config.settings.settings().icon.clone();

            egui::Window::new(format!("Edit Icon: {}", editing_icon))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    let mut current_vendor = icon_settings
                        .active_overrides
                        .get(&editing_icon)
                        .and_then(|o| o.vendor.clone())
                        .unwrap_or_else(|| "default".to_string());
                    let mut current_color = icon_settings
                        .active_overrides
                        .get(&editing_icon)
                        .and_then(|o| o.color_hex.clone())
                        .unwrap_or_else(|| "".to_string());

                    ui.horizontal(|ui| {
                        ui.label("Vendor:");
                        egui::ComboBox::from_id_source("icon_override_vendor")
                            .selected_text(current_vendor.clone())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut current_vendor,
                                    "default".to_string(),
                                    "Default",
                                );
                                for p in crate::icon::AVAILABLE_PACKS {
                                    let pid = p.manifest().id.to_string();
                                    ui.selectable_value(&mut current_vendor, pid.clone(), pid);
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color (Hex):");
                        ui.text_edit_singleline(&mut current_color);
                        if current_color.is_empty() {
                            ui.label("(Default)");
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            let ov = icon_settings
                                .active_overrides
                                .entry(editing_icon.clone())
                                .or_insert_with(|| {
                                    katana_platform::settings::types::icon::IconOverride {
                                        vendor: None,
                                        color_hex: None,
                                    }
                                });
                            ov.vendor = if current_vendor == "default" {
                                None
                            } else {
                                Some(current_vendor)
                            };
                            ov.color_hex = if current_color.is_empty() {
                                None
                            } else {
                                Some(current_color)
                            };

                            // Check if empty, then remove
                            if ov.vendor.is_none() && ov.color_hex.is_none() {
                                icon_settings.active_overrides.remove(&editing_icon);
                            }

                            icon_settings.active_preset = None; // Custom now
                            state.config.settings.settings_mut().icon = icon_settings.clone();
                            let _ = state.config.try_save_settings();

                            close_dialog = true;
                        }
                        if ui.button("Reset").clicked() {
                            icon_settings.active_overrides.remove(&editing_icon);
                            icon_settings.active_preset = None;
                            state.config.settings.settings_mut().icon = icon_settings.clone();
                            let _ = state.config.try_save_settings();
                            close_dialog = true;
                        }
                        if ui.button("Close").clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if close_dialog {
                ui.data_mut(|d| {
                    d.insert_temp(egui::Id::new("katana_editing_icon_id"), String::new())
                });
                // Reinstall pack so overrides are applied to ui.ctx
                crate::icon::IconRegistry::install_pack_by_id(
                    ui.ctx(),
                    &state.config.settings.settings().theme.icon_pack,
                    &state.config.settings.settings().icon,
                );
            }
        }
    }
}
