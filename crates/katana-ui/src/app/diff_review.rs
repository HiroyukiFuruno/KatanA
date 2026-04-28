use crate::app::DocumentOps;
use crate::diff_review::{DiagnosticFixApplicationOps, DiffReviewDecision, DiffReviewState};
use crate::shell::KatanaApp;
use std::path::{Path, PathBuf};

pub(crate) const LINT_FIX_REVIEW_PATH: &str = "Katana://DiffReview/LintFixReview";
const LINT_FIX_REVIEW_PREFIX: &str = "Katana://DiffReview/";

pub(crate) struct LintFixReviewPath;

impl LintFixReviewPath {
    pub(crate) fn is_review_path(path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.starts_with(LINT_FIX_REVIEW_PREFIX)
    }

    pub(crate) fn path() -> PathBuf {
        PathBuf::from(LINT_FIX_REVIEW_PATH)
    }
}

impl KatanaApp {
    fn lint_fix_review_restore_path(&self, first_target_path: Option<&Path>) -> Option<PathBuf> {
        match self.state.active_path() {
            Some(path) if LintFixReviewPath::is_review_path(&path) => self
                .state
                .layout
                .diff_review
                .as_ref()
                .and_then(|review| review.restore_path.clone())
                .or_else(|| first_target_path.map(Path::to_path_buf)),
            Some(path) => Some(path),
            None => first_target_path.map(Path::to_path_buf),
        }
    }
}

pub(crate) trait DiffReviewActionOps {
    fn open_lint_fix_review(&mut self, batches: Vec<crate::app_action::LintFixBatch>);
    fn handle_confirm_current_diff_review_file(&mut self, ctx: &eframe::egui::Context);
    fn handle_reject_current_diff_review_file(&mut self);
    fn handle_reject_all_diff_review_files(&mut self, ctx: &eframe::egui::Context);
}

impl DiffReviewActionOps for KatanaApp {
    fn open_lint_fix_review(&mut self, batches: Vec<crate::app_action::LintFixBatch>) {
        let first_target_path = batches.first().map(|batch| batch.path.clone());
        let original_path = self.lint_fix_review_restore_path(first_target_path.as_deref());
        let mut review_files = Vec::new();

        for batch in batches {
            if batch.fixes.is_empty() {
                continue;
            }
            let target_path = batch.path;
            if let Some(before) = self.load_lint_fix_review_source(&target_path)
                && let Some(file) = DiagnosticFixApplicationOps::build_review_file(
                    target_path,
                    before,
                    &batch.fixes,
                )
            {
                review_files.push(file);
            }
        }

        if review_files.is_empty() {
            return;
        }

        let review_path = LintFixReviewPath::path();
        let mut doc = katana_core::document::Document::new_empty(&review_path);
        doc.is_loaded = true;
        if let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| d.path == review_path)
        {
            self.state.document.open_documents[idx] = doc;
        } else {
            self.state.document.open_documents.push(doc);
        }

        let settings = self.state.config.settings.settings();
        let mode = settings.behavior.diff_view_mode;
        let workspace_root = self
            .state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.clone());
        self.state.layout.diff_review = Some(
            DiffReviewState::new(review_files, mode, original_path)
                .with_workspace_root(workspace_root),
        );

        self.handle_select_document(review_path, true);
    }

    fn handle_confirm_current_diff_review_file(&mut self, ctx: &eframe::egui::Context) {
        let Some(file) = self
            .state
            .layout
            .diff_review
            .as_ref()
            .and_then(|review| review.current_file().cloned())
        else {
            return;
        };

        if !self.apply_diff_review_file_content(ctx, &file) {
            return;
        }
        if let Some(review) = &mut self.state.layout.diff_review {
            review.mark_current(DiffReviewDecision::Applied);
        }
        self.close_diff_review_if_complete();
    }

    fn handle_reject_current_diff_review_file(&mut self) {
        if let Some(review) = &mut self.state.layout.diff_review {
            review.mark_current(DiffReviewDecision::Rejected);
        }
        self.close_diff_review_if_complete();
    }

    fn handle_reject_all_diff_review_files(&mut self, _ctx: &eframe::egui::Context) {
        if let Some(review) = &mut self.state.layout.diff_review {
            review.reject_all_pending();
        }
        self.close_diff_review_if_complete();
    }
}
