/* WHY: Encapsulated main panel rendering logic to manage UI layout complexity and maintain architectural separation. */

use crate::app::download::DownloadOps;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub fn show_main_panels(
        &mut self,
        ctx: &egui::Context,
        theme_colors: &katana_platform::theme::ThemeColors,
    ) -> bool {
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

            egui::CentralPanel::default().show(ctx, |ui| {
                crate::views::modals::terms::TermsModal::new(
                    crate::about_info::APP_VERSION,
                    &mut self.pending_action,
                )
                .show(ui);
            });
            return false;
        }
        let is_blocked = self.is_foreground_surface_active(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                if is_blocked {
                    ui.visuals_mut().disabled_alpha = 1.0;
                }
                ui.set_enabled(!is_blocked);
                let download_req =
                    crate::views::app_frame::MainPanels::new(self, theme_colors).show(ui);
                if let Some(req) = download_req {
                    self.start_download(req);
                }
            });
        true
    }
}
