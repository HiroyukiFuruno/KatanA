use crate::app::doc_search::DocSearchRefresh;
use crate::app::*;
use crate::shell::KatanaApp;

pub(crate) trait DocumentEditOps {
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String);
    fn handle_apply_lint_fixes(
        &mut self,
        fixes: Vec<katana_linter::rules::markdown::DiagnosticFix>,
    );
}

impl DocumentEditOps for KatanaApp {
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];
        doc.buffer.replace_range(span, &replacement);

        use crate::state::document::VirtualPathExt as _;
        if !doc.path.is_virtual_path() {
            doc.is_dirty = true;
        }

        let path = doc.path.clone();
        let content = doc.buffer.clone();
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        self.full_refresh_preview(&path, &content, true, concurrency);

        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
        self.state.diagnostics.last_buffer_update = Some(std::time::Instant::now()); /* WHY: FB32 */
    }

    fn handle_apply_lint_fixes(
        &mut self,
        fixes: Vec<katana_linter::rules::markdown::DiagnosticFix>,
    ) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];
        let next_content =
            crate::app::lint_fix::LintFixApplication::apply_to_content(&doc.buffer, &fixes);

        if next_content != doc.buffer {
            doc.buffer = next_content;
            use crate::state::document::VirtualPathExt as _;
            if !doc.path.is_virtual_path() {
                doc.is_dirty = true;
            }
        }

        let path = doc.path.clone();
        let content = doc.buffer.clone();
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        self.full_refresh_preview(&path, &content, true, concurrency);

        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
        self.handle_action_refresh_diagnostics();
    }
}
