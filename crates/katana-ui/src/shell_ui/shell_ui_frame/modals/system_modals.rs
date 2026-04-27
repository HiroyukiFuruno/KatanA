/* WHY: Isolated system-level modals (About, Settings, Meta, Updates, Slideshow) for clear responsibility. */

use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(crate) fn show_system_modals(&mut self, ctx: &egui::Context) {
        if self.state.layout.show_slideshow {
            if let Some(doc) = self.state.active_document() {
                if let Some(preview) = self.tab_previews.iter_mut().find(|p| p.path == doc.path) {
                    crate::preview_pane::FullscreenLogicOps::render_slideshow_modal(
                        ctx,
                        &mut self.state.layout,
                        &mut preview.pane,
                    );
                } else {
                    self.state.layout.show_slideshow = false;
                }
            } else {
                self.state.layout.show_slideshow = false;
            }
        }

        if let Some(settings_action) =
            crate::settings::SettingsWindow::new(&mut self.state, &mut self.settings_preview)
                .show(ctx)
                .filter(|_| matches!(self.pending_action, AppAction::None))
        {
            self.pending_action = settings_action;
        }

        if let Some(review) = &mut self.state.layout.diff_review
            && let Some(action) = crate::views::modals::diff_review::DiffReviewModal::new(review)
                .show(ctx)
                .filter(|_| matches!(self.pending_action, AppAction::None))
        {
            self.pending_action = action;
        }

        if self.show_about {
            crate::views::modals::about::AboutModal::new(
                &mut self.show_about,
                self.about_icon.as_ref(),
                &mut self.pending_action,
            )
            .show(ctx);
            if matches!(self.pending_action, AppAction::ShowReleaseNotes) {
                self.show_about = false;
            }
        }

        if let Some(path) = self.show_meta_info_for.clone() {
            let mut is_open = true;
            let actual_doc = self
                .state
                .document
                .open_documents
                .iter()
                .find(|d| d.path == path);
            crate::views::modals::meta_info::MetaInfoModal::new(&mut is_open, &path, actual_doc)
                .show(ctx);
            if !is_open {
                self.show_meta_info_for = None;
            }
        }

        if self.show_update_dialog {
            crate::views::modals::update::UpdateModal::new(
                &mut self.show_update_dialog,
                &self.state,
                &mut self.update_markdown_cache,
                &mut self.pending_action,
            )
            .show(ctx);
        }
    }
}
