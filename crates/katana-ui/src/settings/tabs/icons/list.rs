use super::popups::IconsPopupsOps;
use super::table::IconsTableOps;
use crate::settings::*;
use katana_platform::settings::types::icon::IconSettings;

/* WHY: Operations for rendering the main scrollable list of icon settings. */
pub(crate) struct IconsListOps;

impl IconsListOps {
    /* WHY: Renders the vertical scroll area containing the preview grid and advanced settings collapse. */
    pub(crate) fn render(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        icon_settings: &mut IconSettings,
        current_pack: &str,
        settings_changed: &mut bool,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("icon_pack_preview_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                IconsPopupsOps::render_preview_grid(ui, icon_settings, current_pack);

                ui.add_space(SECTION_SPACING);

                let resp = egui::CollapsingHeader::new(&i18n.settings.icons.advanced_settings)
                    .default_open(false)
                    .show(ui, |ui| {
                        super::general::IconsGeneralOps::render(
                            ui,
                            i18n,
                            icon_settings,
                            settings_changed,
                        );

                        ui.add_space(SECTION_SPACING);

                        IconsTableOps::render(ui, state, i18n, icon_settings, settings_changed);
                    });

                let is_advanced_open = resp.body_returned.is_some();
                ui.data_mut(|d| {
                    d.insert_temp(egui::Id::new("icons_advanced_is_open"), is_advanced_open)
                });
            });
    }
}
