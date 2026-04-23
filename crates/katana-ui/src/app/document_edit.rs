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
        doc.is_dirty = true;

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
        self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
    }

    fn handle_apply_lint_fixes(
        &mut self,
        mut fixes: Vec<katana_linter::rules::markdown::DiagnosticFix>,
    ) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];

        /* WHY: Sort fixes descending by start_line and start_column so replacements don't invalidate subsequent offsets */
        fixes.sort_by(|a, b| {
            b.start_line
                .cmp(&a.start_line)
                .then_with(|| b.start_column.cmp(&a.start_column))
        });

        for fix in fixes {
            /* WHY: DiagnosticFix uses 1-indexed line numbers, but
             * line_col_to_byte_index expects 0-indexed lines.
             * Without this conversion, the replacement targets the wrong
             * line, causing duplicate/shifted text (FB29 root cause). */
            let start_opt =
                crate::views::panels::editor::types::EditorLogicOps::line_col_to_byte_index(
                    &doc.buffer,
                    fix.start_line.saturating_sub(1),
                    fix.start_column,
                );
            let end_opt =
                crate::views::panels::editor::types::EditorLogicOps::line_col_to_byte_index(
                    &doc.buffer,
                    fix.end_line.saturating_sub(1),
                    fix.end_column,
                );
            if let (Some(start), Some(end)) = (start_opt, end_opt) {
                let valid = start <= end && end <= doc.buffer.len();
                if valid {
                    doc.buffer.replace_range(start..end, &fix.replacement);
                    doc.is_dirty = true;
                }
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
