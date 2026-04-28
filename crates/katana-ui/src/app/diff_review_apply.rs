use std::path::Path;

use crate::app::DocumentOps;
use crate::app_state::StatusType;
use crate::diff_review::{DiffReviewFile, DiffReviewState};
use crate::shell::KatanaApp;

impl KatanaApp {
    pub(crate) fn load_lint_fix_review_source(&self, path: &Path) -> Option<String> {
        if let Some(doc) = self
            .state
            .document
            .open_documents
            .iter()
            .find(|doc| doc.path == path)
        {
            return Some(doc.buffer.clone());
        }
        self.fs
            .load_document(path.to_path_buf())
            .ok()
            .map(|doc| doc.buffer)
    }

    pub(crate) fn apply_diff_review_file_content(
        &mut self,
        ctx: &eframe::egui::Context,
        file: &DiffReviewFile,
    ) -> bool {
        let Some(idx) = self.find_or_load_diff_review_target_document(&file.path) else {
            return false;
        };
        let before = self.state.document.open_documents[idx].buffer.clone();
        if before != file.before {
            self.state.layout.status_message = Some((
                crate::i18n::I18nOps::get()
                    .diff_review
                    .content_changed
                    .clone(),
                StatusType::Warning,
            ));
            return false;
        }

        self.write_diff_review_content(ctx, idx, file);
        true
    }

    pub(crate) fn close_diff_review_if_complete(&mut self) {
        let is_complete = self
            .state
            .layout
            .diff_review
            .as_ref()
            .is_some_and(DiffReviewState::is_complete);
        if !is_complete {
            return;
        }

        let restore_path = self
            .state
            .layout
            .diff_review
            .as_ref()
            .and_then(|review| review.restore_path.clone());
        self.state.layout.diff_review = None;
        self.remove_open_lint_fix_review_tab_if_any();
        if let Some(path) = restore_path {
            self.handle_select_document(path, true);
        }
    }

    pub(crate) fn close_diff_review_if_tab_removed(&mut self) {
        let Some(review) = self.state.layout.diff_review.as_ref() else {
            return;
        };

        if self
            .state
            .document
            .open_documents
            .iter()
            .any(|doc| crate::app::LintFixReviewPath::is_review_path(&doc.path))
        {
            return;
        }

        let restore_path = review.restore_path.clone();
        self.state.layout.diff_review = None;
        self.state
            .diagnostics
            .remove_file_diagnostics(&crate::app::LintFixReviewPath::path());
        self.state
            .diagnostics
            .remove_file_diagnostics(std::path::Path::new("Katana://DiffReview/lint-fix.md"));
        if let Some(path) = restore_path {
            self.handle_select_document(path, true);
        }
    }
}
