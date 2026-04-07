use super::types::*;
use crate::settings::*;

impl IconsTabOps {
    pub(crate) fn render_icons_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        SettingsOps::section_header(ui, &crate::i18n::I18nOps::get().settings.theme.icon_pack);

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
                ui.horizontal_wrapped(|ui| {
                    const ITEM_SPACING: f32 = 16.0;
                    ui.spacing_mut().item_spacing = egui::vec2(ITEM_SPACING, ITEM_SPACING);
                    for icon in crate::icon::ALL_ICONS {
                        let response = ui.add(icon.ui_image(ui, crate::icon::IconSize::Large));
                        response.on_hover_text(icon.name());
                    }
                });
            });
    }
}
