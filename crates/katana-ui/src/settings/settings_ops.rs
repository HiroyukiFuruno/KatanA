use super::types::SettingsOps;
use eframe::egui;

impl SettingsOps {
    pub(crate) fn render_settings_tree(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        super::settings_tree::render_settings_tree(ui, state);
    }

    pub(crate) fn section_header(ui: &mut egui::Ui, text: &str) {
        super::settings_helpers::section_header(ui, text);
    }

    pub(crate) fn add_styled_slider<'a>(
        ui: &mut egui::Ui,
        slider: egui::Slider<'a>,
    ) -> egui::Response {
        super::settings_helpers::add_styled_slider(ui, slider)
    }
}
