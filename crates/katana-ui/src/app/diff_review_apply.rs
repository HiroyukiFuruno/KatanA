use std::path::Path;

use crate::app::{DocumentOps, PreviewOps, doc_search::DocSearchRefresh};
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
        self.handle_select_document(file.path.clone(), true);
        if self.state.active_path().as_ref() != Some(&file.path) {
            return false;
        }

        let Some(idx) = self.state.document.active_doc_idx else {
            return false;
        };
        if self.state.document.open_documents[idx].buffer != file.before {
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

    fn write_diff_review_content(
        &mut self,
        ctx: &eframe::egui::Context,
        idx: usize,
        file: &DiffReviewFile,
    ) {
        let path = file.path.clone();
        let before = file.before.clone();
        let content = file.after.clone();
        {
            let doc = &mut self.state.document.open_documents[idx];
            doc.buffer = content.clone();
            use crate::state::document::VirtualPathExt as _;
            if !doc.path.is_virtual_path() {
                doc.is_dirty = true;
            }
        }

        self.record_diff_review_undo(ctx, &path, &before, &content);
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        self.full_refresh_preview(&path, &content, true, concurrency);

        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
        self.handle_action_refresh_diagnostics();
    }

    fn record_diff_review_undo(
        &self,
        ctx: &eframe::egui::Context,
        path: &Path,
        before: &str,
        after: &str,
    ) {
        let workspace_root = self
            .state
            .workspace
            .data
            .as_ref()
            .map(|ws| ws.root.as_path());
        crate::editor_undo::EditorUndoOps::record_external_change(
            ctx,
            workspace_root,
            path,
            before,
            after,
        );
    }

    pub(crate) fn close_diff_review_if_complete(&mut self) {
        let restore_path = self
            .state
            .layout
            .diff_review
            .as_ref()
            .filter(|review| review.is_complete())
            .and_then(|review| review.restore_path.clone());
        let is_complete = self
            .state
            .layout
            .diff_review
            .as_ref()
            .is_some_and(DiffReviewState::is_complete);
        if !is_complete {
            return;
        }
        self.state.layout.diff_review = None;
        if let Some(path) = restore_path {
            self.handle_select_document(path, true);
        }
    }
}
