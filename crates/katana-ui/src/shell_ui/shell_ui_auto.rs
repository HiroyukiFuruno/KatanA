use crate::app::PreviewOps;
use crate::shell::KatanaApp;
use eframe::egui;

const MIN_DEFERRED_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

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
        if self
            .state
            .active_document()
            .is_some_and(|doc| katana_core::workspace::TreeEntry::path_is_html(&doc.path))
        {
            if behavior.auto_refresh {
                ctx.request_repaint_after(MIN_DEFERRED_REFRESH_INTERVAL);
            }
            return;
        }
        if let Some(interval) =
            deferred_refresh_interval(behavior.auto_refresh, behavior.auto_refresh_interval_secs)
        {
            let now = std::time::Instant::now();
            match self.state.document.last_auto_refresh {
                Some(last) if now.duration_since(last) >= interval => {
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
            ctx.request_repaint_after(interval);
        } else {
            self.state.document.last_auto_refresh = None;
        }
    }

    pub(super) fn tick_pending_html_preview_refresh(&mut self) {
        let Some(pending) = self.pending_html_preview_refresh.take() else {
            return;
        };
        if pending.due_at > std::time::Instant::now() {
            self.pending_html_preview_refresh = Some(pending);
            return;
        }
        let Some(doc) = self.state.active_document() else {
            return;
        };
        if doc.path != pending.path || !katana_core::workspace::TreeEntry::path_is_html(&doc.path) {
            return;
        }
        let path = doc.path.clone();
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "HTML preview refresh read failed");
                return;
            }
        };
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        self.full_refresh_preview(&path, &content, true, concurrency);
    }
}

fn deferred_refresh_interval(enabled: bool, interval_secs: f64) -> Option<std::time::Duration> {
    if !enabled || !interval_secs.is_finite() || interval_secs < 0.0 {
        return None;
    }
    Some(std::time::Duration::from_secs_f64(interval_secs).max(MIN_DEFERRED_REFRESH_INTERVAL))
}

#[cfg(test)]
mod tests {
    use super::{MIN_DEFERRED_REFRESH_INTERVAL, deferred_refresh_interval};

    #[test]
    fn auto_refresh_defers_zero_interval_without_disabling_it() {
        assert_eq!(
            deferred_refresh_interval(true, 0.0),
            Some(MIN_DEFERRED_REFRESH_INTERVAL)
        );
        assert_eq!(
            deferred_refresh_interval(true, 0.1),
            Some(MIN_DEFERRED_REFRESH_INTERVAL)
        );
        assert_eq!(
            deferred_refresh_interval(true, 2.0),
            Some(std::time::Duration::from_secs(2))
        );
        assert_eq!(deferred_refresh_interval(true, -1.0), None);
        assert_eq!(deferred_refresh_interval(false, 2.0), None);
    }
}
