use crate::diff_review::{DiagnosticFixApplicationOps, DiffReviewDecision, DiffReviewState};
use crate::shell::KatanaApp;

pub(crate) trait DiffReviewActionOps {
    fn open_lint_fix_review(&mut self, batches: Vec<crate::app_action::LintFixBatch>);
    fn handle_confirm_current_diff_review_file(&mut self, ctx: &eframe::egui::Context);
    fn handle_reject_current_diff_review_file(&mut self);
    fn handle_reject_all_diff_review_files(&mut self, ctx: &eframe::egui::Context);
}

impl DiffReviewActionOps for KatanaApp {
    fn open_lint_fix_review(&mut self, batches: Vec<crate::app_action::LintFixBatch>) {
        let original_path = self.state.active_path();
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
