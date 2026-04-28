use std::path::Path;

use crate::app::{PreviewOps, WorkspaceOps, doc_search::DocSearchRefresh};
use crate::diff_review::DiffReviewFile;
use crate::shell::KatanaApp;

impl KatanaApp {
    pub(crate) fn find_or_load_diff_review_target_document(
        &mut self,
        path: &Path,
    ) -> Option<usize> {
        if let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|doc| doc.path == path)
        {
            return Some(idx);
        }

        let doc = self.fs.load_document(path.to_path_buf()).ok()?;
        self.state.document.open_documents.push(doc);
        self.state.initialize_tab_split_state(path);
        self.state.document.open_documents.len().checked_sub(1)
    }

    pub(crate) fn write_diff_review_content(
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

    pub(crate) fn record_diff_review_undo(
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

    pub(crate) fn remove_open_lint_fix_review_tab_if_any(&mut self) {
        let review_path = crate::app::LintFixReviewPath::path();
        let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|doc| crate::app::LintFixReviewPath::is_review_path(&doc.path))
        else {
            return;
        };

        let removed_path = self.state.document.open_documents.remove(idx).path;
        let path_string = review_path.to_string_lossy().to_string();
        let removed_path_string = removed_path.to_string_lossy().to_string();
        for group in &mut self.state.document.tab_groups {
            group
                .members
                .retain(|member| member != &path_string && member != &removed_path_string);
        }
        self.state
            .document
            .tab_groups
            .retain(|group| !group.members.is_empty());
        self.state
            .document
            .tab_view_modes
            .retain(|mode| !crate::app::LintFixReviewPath::is_review_path(&mode.path));
        self.state
            .document
            .tab_split_states
            .retain(|state| !crate::app::LintFixReviewPath::is_review_path(&state.path));
        self.state.document.active_doc_idx = if self.state.document.open_documents.is_empty() {
            None
        } else {
            let mut active_idx = self.state.document.active_doc_idx.unwrap_or(0);
            if idx <= active_idx {
                active_idx = active_idx.saturating_sub(1);
            }
            Some(active_idx.min(self.state.document.open_documents.len() - 1))
        };
        self.state.diagnostics.remove_file_diagnostics(&review_path);
        self.state
            .diagnostics
            .remove_file_diagnostics(&removed_path);
        self.state.document.cleanup_empty_groups();
        self.save_workspace_state();
    }
}
