/* WHY: Encapsulated main panel rendering logic to manage UI layout complexity and maintain architectural separation. */

use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub fn show_main_panels(
        &mut self,
        ui: &mut egui::Ui,
        theme_colors: &katana_platform::theme::ThemeColors,
    ) -> bool {
        let ctx = ui.ctx().clone();
        let terms_accepted = self
            .state
            .config
            .settings
            .settings()
            .terms_accepted_version
            .is_some();

        if !terms_accepted {
            self.show_update_dialog = false;
            self.needs_changelog_display = false;

            egui::CentralPanel::default().show(ui, |ui| {
                crate::views::modals::terms::TermsModal::new(
                    crate::about_info::APP_VERSION,
                    &mut self.pending_action,
                )
                .show(ui);
            });
            return false;
        }
        let is_blocked = self.is_foreground_surface_active(&ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0.0))
            .show(ui, |ui| {
                crate::widgets::InteractionFacade::scope(ui, is_blocked, |ui| {
                    crate::views::app_frame::MainPanels::new(self, theme_colors).show(ui);
                });
            });
        true
    }
}
