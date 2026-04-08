/* WHY: Operations for rendering icon-related popups and the preview grid. */
pub(crate) struct IconsPopupsOps;

impl IconsPopupsOps {
    /* WHY: Renders non-modal windows like the Preset Save dialog. */
    pub(crate) fn render(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let i18n = crate::i18n::I18nOps::get();
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

            egui::Window::new(&i18n.settings.icons.save_preset)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(&i18n.settings.icons.preset_name);
                    let response = ui.text_edit_singleline(&mut preset_name);
                    response.request_focus();
                    crate::widgets::AlignCenter::new()
                        .content(|ui| {
                            if ui.button(&i18n.action.save).clicked() {
                                let mut icon_settings =
                                    state.config.settings.settings().icon.clone();
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
                            if ui.button(&i18n.action.cancel).clicked() {
                                close_dialog = true;
                            }
                        })
                        .show(ui);
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
    }

    /* WHY: Renders a collapsible grid of all available icons grouped by vendor. */
    pub(crate) fn render_preview_grid(
        ui: &mut egui::Ui,
        icon_settings: &katana_platform::settings::types::icon::IconSettings,
        current_pack: &str,
    ) {
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
                    Self::render_vendor_icons(ui, icons, icon_settings, current_pack);
                });
        }
    }

    /* WHY: Renders a wrapping horizontal layout of icons for a specific vendor. */
    fn render_vendor_icons(
        ui: &mut egui::Ui,
        icons: Vec<&crate::icon::Icon>,
        icon_settings: &katana_platform::settings::types::icon::IconSettings,
        current_pack: &str,
    ) {
        use crate::theme_bridge::ThemeBridgeOps;

        /* WHY: allow(horizontal_layout) */
        ui.horizontal_wrapped(|ui| {
            const ITEM_SPACING: f32 = 16.0;
            ui.spacing_mut().item_spacing = egui::vec2(ITEM_SPACING, ITEM_SPACING);
            for icon in icons {
                let image = icon.image(crate::icon::IconSize::Large);

                let mut color = ui.visuals().text_color();

                let current_ov = icon_settings.active_overrides.get(icon.name());
                let current_color = current_ov.and_then(|o| o.color);
                let current_frame_color = current_ov.and_then(|o| o.frame_color);

                if icon_settings.colorful_vendor_icons {
                    if let Some(rgba) = current_color {
                        color = ThemeBridgeOps::rgba_to_color32(rgba);
                    } else {
                        color = icon
                            .vendor_default_color(current_pack, ui.visuals().dark_mode)
                            .unwrap_or(color);
                    }
                }

                let icon_bg = if ui.visuals().dark_mode {
                    crate::theme_bridge::TRANSPARENT
                } else {
                    ThemeBridgeOps::light_mode_icon_bg()
                };
                let mut btn = egui::Button::image(image.tint(color))
                    .frame(false)
                    .fill(icon_bg);
                if let Some(rgba) =
                    current_frame_color.filter(|_| icon_settings.colorful_vendor_icons)
                {
                    btn = btn.frame(true).fill(ThemeBridgeOps::rgba_to_color32(rgba));
                }

                let response = ui.add(btn);
                response.on_hover_text(icon.name());
            }
        });
    }
}
