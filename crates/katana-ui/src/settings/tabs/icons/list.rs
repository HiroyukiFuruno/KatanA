use super::popups::IconsPopupsOps;
use katana_platform::settings::types::icon::IconSettings;

/* WHY: Operations for rendering the main scrollable list of icon settings. */
pub(crate) struct IconsListOps;

impl IconsListOps {
    /* WHY: Renders the vertical scroll area containing the preview grid and advanced settings collapse. */
    pub(crate) fn render(
        ui: &mut egui::Ui,
        _state: &mut crate::app_state::AppState,
        _i18n: &crate::i18n::I18nMessages,
        icon_settings: &mut IconSettings,
        current_pack: &str,
        _settings_changed: &mut bool,
    ) {
        IconsPopupsOps::render_preview_grid(ui, icon_settings, current_pack);
    }
}
