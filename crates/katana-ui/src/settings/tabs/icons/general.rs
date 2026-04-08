use katana_platform::settings::types::icon::IconSettings;

/* WHY: Operations for the general icon settings section (e.g. toggle for colorful icons). */
pub(crate) struct IconsGeneralOps;

impl IconsGeneralOps {
    /* WHY: Renders the general configuration checkboxes. */
    pub(crate) fn render(
        ui: &mut egui::Ui,
        i18n: &crate::i18n::I18nMessages,
        icon_settings: &mut IconSettings,
        settings_changed: &mut bool,
    ) {
        const SPACING_SMALL: f32 = 8.0;
        ui.add_space(SPACING_SMALL);

        if ui
            .checkbox(
                &mut icon_settings.colorful_vendor_icons,
                &i18n.settings.icons.colorful_vendor_icons_label,
            )
            .changed()
        {
            *settings_changed = true;
        }
    }
}
