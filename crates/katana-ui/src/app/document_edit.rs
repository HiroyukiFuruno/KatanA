use crate::app::doc_search::DocSearchRefresh;
use crate::app::*;
use crate::shell::KatanaApp;

pub(crate) trait DocumentEditOps {
    fn handle_replace_text(
        &mut self,
        ctx: &eframe::egui::Context,
        span: std::ops::Range<usize>,
        replacement: String,
    );
    fn handle_apply_lint_fixes(
        &mut self,
        ctx: &eframe::egui::Context,
        fixes: Vec<katana_markdown_linter::rules::markdown::DiagnosticFix>,
    );
    fn handle_apply_lint_fixes_for_files(
        &mut self,
        ctx: &eframe::egui::Context,
        batches: Vec<crate::app_action::LintFixBatch>,
    );
}

impl DocumentEditOps for KatanaApp {
    fn handle_replace_text(
        &mut self,
        ctx: &eframe::egui::Context,
        span: std::ops::Range<usize>,
        replacement: String,
    ) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];
        let before = doc.buffer.clone();
        doc.buffer.replace_range(span, &replacement);

        use crate::state::document::VirtualPathExt as _;
        if !doc.path.is_virtual_path() {
            doc.is_dirty = true;
        }

        let path = doc.path.clone();
        let content = doc.buffer.clone();
        let workspace_root = self
            .state
            .workspace
            .data
            .as_ref()
            .map(|ws| ws.root.as_path());
        crate::editor_undo::EditorUndoOps::record_external_change(
            ctx,
            workspace_root,
            &path,
            &before,
            &content,
        );
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
        self.state.diagnostics.last_buffer_update = Some(std::time::Instant::now()); /* WHY: FB32 */
    }

    fn handle_apply_lint_fixes(
        &mut self,
        _ctx: &eframe::egui::Context,
        fixes: Vec<katana_markdown_linter::rules::markdown::DiagnosticFix>,
    ) {
        let Some(path) = self.state.active_path() else {
            return;
        };
        self.handle_apply_lint_fixes_for_files(
            _ctx,
            vec![crate::app_action::LintFixBatch { path, fixes }],
        );
    }

    fn handle_apply_lint_fixes_for_files(
        &mut self,
        _ctx: &eframe::egui::Context,
        batches: Vec<crate::app_action::LintFixBatch>,
    ) {
        self.open_lint_fix_review(batches);
    }
}
