use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(super) fn tick_auto_save(&mut self) {
        let behavior = self.state.config.settings.settings().behavior.clone();
        if behavior.auto_save && behavior.auto_save_interval_secs >= 0.0 {
            let now = std::time::Instant::now();
            match self.state.document.last_auto_save {
                Some(last)
                    if now.duration_since(last).as_secs_f64()
                        >= behavior.auto_save_interval_secs =>
                {
                    if self.state.active_document().is_some_and(|d| d.is_dirty)
                        && matches!(self.pending_action, crate::app_state::AppAction::None)
                    {
                        self.pending_action = crate::app_state::AppAction::SaveDocument;
                    }
                    self.state.document.last_auto_save = Some(now);
                }
                None => self.state.document.last_auto_save = Some(now),
                _ => {}
            }
        } else {
            self.state.document.last_auto_save = None;
        }
    }

    pub(super) fn tick_auto_refresh(&mut self, ctx: &egui::Context) {
        let behavior = self.state.config.settings.settings().behavior.clone();
        if behavior.auto_refresh && behavior.auto_refresh_interval_secs >= 0.0 {
            let now = std::time::Instant::now();
            match self.state.document.last_auto_refresh {
                Some(last)
                    if now.duration_since(last).as_secs_f64()
                        >= behavior.auto_refresh_interval_secs =>
                {
                    if self.state.active_document().is_some()
                        && matches!(self.pending_action, crate::app_state::AppAction::None)
                    {
                        self.pending_action =
                            crate::app_state::AppAction::RefreshDocument { is_manual: false };
                    }
                    self.state.document.last_auto_refresh = Some(now);
                }
                None => self.state.document.last_auto_refresh = Some(now),
                _ => {}
            }
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(
                behavior.auto_refresh_interval_secs,
            ));
        } else {
            self.state.document.last_auto_refresh = None;
        }
    }
}
