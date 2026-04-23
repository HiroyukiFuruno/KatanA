pub use super::types::IconsTabOps;
use crate::settings::*;
use crate::widgets::AlignCenter;

pub mod colors;
mod general;
pub mod list;
pub mod panels;
pub mod popups;
pub mod row;
pub mod table;
mod table_row;

impl IconsTabOps {
    /* WHY: Renders the primary entry point for the Icon settings tab. */
    pub(crate) fn render_icons_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let i18n = crate::i18n::I18nOps::get();
        popups::IconsPopupsOps::render(ui, state);

        let mut current_pack = state.config.settings.settings().theme.icon_pack.clone();
        let mut icon_settings = state.config.settings.settings().icon.clone();
        let mut settings_changed = false;

        let mut is_advanced_open = ui
            .data(|d| d.get_temp::<bool>(egui::Id::new("icons_advanced_is_open")))
            .unwrap_or(false);

        if is_advanced_open {
            /* WHY: When advanced panel is open, it takes 100% of the tab height.
             * Skip ComboBox and icon list rendering entirely. */
            crate::settings::tabs::icons::panels::IconsPanelsOps::render_panels(
                ui,
                state,
                i18n,
                &mut is_advanced_open,
                &mut icon_settings,
                &mut settings_changed,
            );
        } else {
            /* WHY: Normal view — ComboBox + icon list + "Advanced Settings" button at bottom. */
            Self::render_normal_view(
                ui,
                state,
                i18n,
                &mut current_pack,
                &mut icon_settings,
                &mut settings_changed,
                &mut is_advanced_open,
            );
        }

        ui.data_mut(|d| d.insert_temp(egui::Id::new("icons_advanced_is_open"), is_advanced_open));

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

    /* WHY: Public test entry point so integration tests can call render_icons_tab directly.
     * Intentionally always-compiled (not cfg(test)) because integration tests are external crates
     * and cannot see cfg(test) symbols from the library. */
    pub fn render_icons_tab_for_test(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        Self::render_icons_tab(ui, state);
    }

    /* WHY: Draws the normal (closed) view — pack selector, icon preview list, and trigger button. */
    fn render_normal_view(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        current_pack: &mut String,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
        is_advanced_open: &mut bool,
    ) {
        let available_packs = [
            ("katana", "KatanA (Default)"),
            ("material-symbols", "Material Symbols"),
            ("lucide", "Lucide"),
            ("tabler-icons", "Tabler Icons"),
            ("heroicons", "Heroicons"),
            ("feather", "Feather"),
        ];

        let selected_name = if let Some(preset_name) = &icon_settings.active_preset {
            preset_name.clone()
        } else {
            available_packs
                .iter()
                .find(|(id, _)| *id == *current_pack)
                .map(|(_, name)| name.to_string())
                .unwrap_or_else(|| current_pack.clone())
        };

        AlignCenter::new()
            .content(|ui| {
                egui::ComboBox::from_id_source("icon_pack_combobox")
                    .selected_text(selected_name.clone())
                    .show_ui(ui, |ui| {
                        for (id, display_name) in available_packs.iter() {
                            let is_selected =
                                icon_settings.active_preset.is_none() && *current_pack == *id;
                            let response = ui.add(
                                egui::Button::selectable(is_selected, *display_name)
                                    .frame_when_inactive(true),
                            );
                            if response.clicked() {
                                *current_pack = id.to_string();
                                icon_settings.active_preset = None;
                                icon_settings.active_overrides.clear();
                                *settings_changed = true;
                            }
                        }

                        if !icon_settings.custom_presets.is_empty() {
                            ui.separator();
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
                                    *settings_changed = true;
                                }
                            }
                        }
                    });
            })
            .show(ui);

        if *current_pack != state.config.settings.settings().theme.icon_pack {
            state.config.settings.settings_mut().theme.icon_pack = current_pack.clone();
            crate::icon::IconRegistry::install_pack_by_id(
                ui.ctx(),
                current_pack,
                &state.config.settings.settings().icon,
            );
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SECTION_SPACING);

        /* WHY: "Advanced Settings" button pinned at the bottom of the tab. */
        const BOTTOM_PANEL_MARGIN_Y: f32 = 8.0;
        egui::TopBottomPanel::bottom("icon_settings_advanced_button_panel")
            .frame(egui::Frame::none().inner_margin(egui::vec2(0.0, BOTTOM_PANEL_MARGIN_Y)))
            .show_inside(ui, |ui| {
                AlignCenter::new()
                    .content(|ui| {
                        if ui
                            .button(&i18n.common.advanced_settings)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            *is_advanced_open = true;
                        }
                    })
                    .show(ui);
            });

        /* WHY: Icon preview grid fills remaining space between ComboBox and button. */
        list::IconsListOps::render(
            ui,
            state,
            i18n,
            icon_settings,
            current_pack,
            settings_changed,
        );
    }
}
