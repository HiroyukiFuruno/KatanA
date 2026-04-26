use std::path::{Path, PathBuf};

use crate::app::doc_search::DocSearchRefresh;
use crate::app::*;
use crate::markdown_formatting_bridge::{
    MarkdownFormatFailure, MarkdownFormattingBridgeOps, MarkdownFormattingSummary,
};
use crate::shell::*;

impl KatanaApp {
    pub(crate) fn handle_action_format_markdown_file(&mut self, path: PathBuf) {
        let summary = self.format_markdown_files(vec![path]);
        self.show_markdown_format_summary(summary);
    }

    pub(crate) fn handle_action_format_workspace_markdown(&mut self, root: PathBuf) {
        let paths = self.collect_workspace_markdown_paths(&root);
        let summary = self.format_markdown_files(paths);
        self.show_markdown_format_summary(summary);
    }

    fn format_markdown_files(&mut self, paths: Vec<PathBuf>) -> MarkdownFormattingSummary {
        let mut summary = MarkdownFormattingSummary::default();
        for path in paths {
            match self.format_markdown_path(&path) {
                Ok(true) => summary.changed_files += 1,
                Ok(false) => summary.unchanged_files += 1,
                Err(failure) => summary.failures.push(failure),
            }
        }
        summary
    }

    fn format_markdown_path(&mut self, path: &Path) -> Result<bool, MarkdownFormatFailure> {
        if !Self::is_markdown_path(path) {
            let message = crate::i18n::I18nOps::get()
                .status
                .format_markdown_not_markdown
                .clone();
            return Err(MarkdownFormatFailure::new(path, message));
        }
        let content = self.load_format_source(path)?;
        let outcome = MarkdownFormattingBridgeOps::format_content(&self.state, path, &content)?;
        if outcome.content == content {
            self.refresh_formatted_diagnostics(path.to_path_buf(), &content);
            return Ok(false);
        }

        self.save_formatted_content(path, outcome.content)?;
        Ok(true)
    }

    fn load_format_source(&self, path: &Path) -> Result<String, MarkdownFormatFailure> {
        if let Some(idx) = self.open_document_index(path) {
            return Ok(self.state.document.open_documents[idx].buffer.clone());
        }
        self.fs
            .load_document(path.to_path_buf())
            .map(|doc| doc.buffer)
            .map_err(|err| MarkdownFormatFailure::new(path, err.to_string()))
    }

    fn save_formatted_content(
        &mut self,
        path: &Path,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        if let Some(idx) = self.open_document_index(path) {
            self.save_open_formatted_document(idx, content)
        } else {
            self.save_unopened_formatted_document(path, content)
        }
    }

    fn save_open_formatted_document(
        &mut self,
        idx: usize,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        let path = self.state.document.open_documents[idx].path.clone();
        let is_active = self.state.document.active_doc_idx == Some(idx);
        {
            let doc = &mut self.state.document.open_documents[idx];
            doc.buffer = content.clone();
            doc.is_dirty = true;
            self.fs
                .save_document(doc)
                .map_err(|err| MarkdownFormatFailure::new(&path, err.to_string()))?;
        }
        self.refresh_after_format(path, &content, is_active);
        Ok(())
    }

    fn save_unopened_formatted_document(
        &mut self,
        path: &Path,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        let mut doc = katana_core::document::Document::new(path, content.clone());
        self.fs
            .save_document(&mut doc)
            .map_err(|err| MarkdownFormatFailure::new(path, err.to_string()))?;
        self.refresh_formatted_diagnostics(path.to_path_buf(), &content);
        Ok(())
    }

    fn refresh_after_format(&mut self, path: PathBuf, content: &str, is_active: bool) {
        self.refresh_formatted_diagnostics(path.clone(), content);
        if !is_active {
            return;
        }
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        self.full_refresh_preview(&path, content, true, concurrency);
        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(content);
        }
    }

    fn refresh_formatted_diagnostics(&mut self, path: PathBuf, content: &str) {
        let diagnostics = crate::linter_bridge::MarkdownLinterBridgeOps::evaluate_document(
            &self.state,
            &path,
            content,
        );
        self.state.diagnostics.update_diagnostics(path, diagnostics);
    }

    fn collect_workspace_markdown_paths(&self, root: &Path) -> Vec<PathBuf> {
        let Some(workspace) = &self.state.workspace.data else {
            return Vec::new();
        };
        if !root.starts_with(&workspace.root) {
            return Vec::new();
        }
        workspace
            .collect_all_markdown_file_paths()
            .into_iter()
            .filter(|path| path.starts_with(root))
            .filter(|path| {
                !Self::is_inside_ignored_directory(
                    &workspace.root,
                    path,
                    &self
                        .state
                        .config
                        .settings
                        .settings()
                        .workspace
                        .ignored_directories,
                )
            })
            .collect()
    }

    fn is_inside_ignored_directory(
        root: &Path,
        path: &Path,
        ignored_directories: &[String],
    ) -> bool {
        let Ok(relative_path) = path.strip_prefix(root) else {
            return false;
        };
        relative_path.components().any(|component| {
            let std::path::Component::Normal(name) = component else {
                return false;
            };
            ignored_directories
                .iter()
                .any(|ignored| name == std::ffi::OsStr::new(ignored.as_str()))
        })
    }

    fn open_document_index(&self, path: &Path) -> Option<usize> {
        self.state
            .document
            .open_documents
            .iter()
            .position(|doc| doc.path == path)
    }

    fn is_markdown_path(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
            })
    }

    fn show_markdown_format_summary(&mut self, summary: MarkdownFormattingSummary) {
        self.state.layout.status_message = Some((summary.status_message(), summary.status_type()));
    }
}
