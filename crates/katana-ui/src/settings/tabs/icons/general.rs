use katana_platform::settings::types::icon::IconSettings;

/* WHY: Operations for the general icon settings section (e.g. toggle for colorful icons). */
pub(crate) struct IconsGeneralOps;

impl IconsGeneralOps {
    /* WHY: Renders the general icon configuration toggles. */
    pub(crate) fn render(
        ui: &mut egui::Ui,
        i18n: &crate::i18n::I18nMessages,
        icon_settings: &mut IconSettings,
        settings_changed: &mut bool,
    ) {
        const SPACING_SMALL: f32 = 8.0;
        ui.add_space(SPACING_SMALL);

        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &i18n.settings.icons.colorful_vendor_icons_label,
                    &mut icon_settings.colorful_vendor_icons,
                )
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            *settings_changed = true;
        }
    }
}
