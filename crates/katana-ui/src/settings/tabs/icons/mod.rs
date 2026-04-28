pub use super::types::IconsTabOps;
use crate::settings::*;

pub mod colors;
mod general;
pub mod list;
pub mod panels;
pub mod popups;
#[cfg(test)]
mod popups_tests;
mod preset_controls;
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
                &current_pack,
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
        preset_controls::IconPresetControlsOps::render(
            ui,
            current_pack,
            icon_settings,
            settings_changed,
            is_advanced_open,
        );
        general::IconsGeneralOps::render(ui, i18n, icon_settings, settings_changed);

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

        /* WHY: Icon preview grid follows the shared preset controls in the normal view. */
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
