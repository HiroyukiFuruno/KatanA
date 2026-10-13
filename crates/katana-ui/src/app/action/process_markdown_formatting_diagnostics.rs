use std::path::PathBuf;

pub(super) struct MarkdownFormattingDiagnosticsOps;

impl MarkdownFormattingDiagnosticsOps {
    pub(super) fn refresh(state: &mut crate::app_state::AppState, path: PathBuf, content: &str) {
        let options =
            crate::linter_options_bridge::MarkdownLinterOptionsBridgeOps::load_effective_options_for_content(
                state,
                &path,
                content,
            );
        let diagnostic_context_hash =
            crate::linter_diagnostic_context::LinterDiagnosticContextOps::hash(state, &options);
        let diagnostics =
            crate::linter_bridge::MarkdownLinterBridgeOps::evaluate_document_with_options(
                state, &path, content, options,
            );
        state.diagnostics.update_diagnostics_for_context(
            path,
            content,
            diagnostic_context_hash,
            diagnostics,
        );
    }
}
