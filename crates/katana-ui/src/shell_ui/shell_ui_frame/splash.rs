/* WHY: Specialized logic for splash screen rendering to maintain UI modularity and architectural clarity. */

use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub fn show_splash(&mut self, ctx: &egui::Context) {
        if let Some(start) = self.splash_start {
            let elapsed = start.elapsed().as_secs_f32();
            let is_loading = self.state.workspace.is_loading;
            let dismissed = crate::views::splash::SplashOverlay::new(
                elapsed,
                self.about_icon.as_ref(),
                is_loading,
            )
            .show(ctx);
            if dismissed {
                ctx.request_repaint();
                self.splash_start = None;
            }
        }
    }
}
