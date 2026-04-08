use super::types::*;
use crate::settings::*;

impl IconsTabOps {
    pub(crate) fn render_icons_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
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
            crate::icon::IconRegistry::install_pack_by_id(ui.ctx(), &current_pack);
            let _ = state.config.try_save_settings();
        }

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
                                    let color = icon
                                        .vendor_default_color(ui.visuals().dark_mode)
                                        .unwrap_or(ui.visuals().text_color());
                                    let response = ui.add(image.tint(color));
                                    response.on_hover_text(icon.name());
                                }
                            });
                        });
                }
            });
    }
}
