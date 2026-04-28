use crate::shell::*;

impl KatanaApp {
    pub(crate) fn handle_action_refresh_diagnostics(&mut self) {
        for (path, content) in self.lintable_open_document_sources() {
            if self.state.diagnostics.is_current(&path, &content) {
                continue;
            }

            let diagnostics = crate::linter_bridge::MarkdownLinterBridgeOps::evaluate_document(
                &self.state,
                path.as_path(),
                &content,
            );
            self.state
                .diagnostics
                .update_diagnostics_for_content(path, &content, diagnostics);
        }
    }

    fn lintable_open_document_sources(&self) -> Vec<(std::path::PathBuf, String)> {
        self.state
            .document
            .open_documents
            .iter()
            .filter_map(|doc| self.lintable_open_document_source(doc))
            .collect()
    }

    fn lintable_open_document_source(
        &self,
        doc: &katana_core::document::Document,
    ) -> Option<(std::path::PathBuf, String)> {
        if !Self::is_lintable_markdown_path(&doc.path) {
            return None;
        }

        if doc.is_loaded {
            return Some((doc.path.clone(), doc.buffer.clone()));
        }

        self.fs
            .load_document(doc.path.clone())
            .ok()
            .map(|loaded| (loaded.path, loaded.buffer))
    }

    fn is_lintable_markdown_path(path: &std::path::Path) -> bool {
        use crate::state::document::VirtualPathExt as _;
        if path.is_virtual_path() {
            let path_str = path.to_string_lossy();
            return path_str.starts_with("Katana://Demo/")
                && (path_str.ends_with("lint-fix.md") || path_str.ends_with("lint-fix.ja.md"));
        }

        path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false)
    }
}
