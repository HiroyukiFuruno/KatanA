use std::path::{Path, PathBuf};

use crate::app::doc_search::DocSearchRefresh;
use crate::app::*;
use crate::markdown_formatting_bridge::{
    MarkdownFormatFailure, MarkdownFormattingBridgeOps, MarkdownFormattingSummary,
};
use crate::shell::*;

use super::process_markdown_formatting_paths::{
    MarkdownFormattingPathFailureOps, MarkdownFormattingPathOps,
};

impl KatanaApp {
    pub(crate) fn handle_action_format_markdown_file(
        &mut self,
        ctx: &eframe::egui::Context,
        path: PathBuf,
    ) {
        let summary = self.format_markdown_files(ctx, vec![path]);
        self.show_markdown_format_summary(summary);
    }

    pub(crate) fn handle_action_format_workspace_markdown(
        &mut self,
        ctx: &eframe::egui::Context,
        root: PathBuf,
    ) {
        let paths = self.collect_workspace_markdown_paths(&root);
        let summary = self.format_markdown_files(ctx, paths);
        self.show_markdown_format_summary(summary);
    }

    fn format_markdown_files(
        &mut self,
        ctx: &eframe::egui::Context,
        paths: Vec<PathBuf>,
    ) -> MarkdownFormattingSummary {
        let mut summary = MarkdownFormattingSummary::default();
        for path in paths {
            match self.format_markdown_path(ctx, &path) {
                Ok(true) => summary.changed_files += 1,
                Ok(false) => summary.unchanged_files += 1,
                Err(failure) => summary.failures.push(failure),
            }
        }
        summary
    }

    fn format_markdown_path(
        &mut self,
        ctx: &eframe::egui::Context,
        path: &Path,
    ) -> Result<bool, MarkdownFormatFailure> {
        if !Self::is_markdown_path(path) {
            return Err(MarkdownFormattingPathFailureOps::markdown_path_failure(
                path,
            ));
        }
        let content = self.load_format_source(path)?;
        let outcome = MarkdownFormattingBridgeOps::format_content(&self.state, path, &content)?;
        if outcome.content == content {
            self.refresh_formatted_diagnostics(path.to_path_buf(), &content);
            return Ok(false);
        }

        self.save_formatted_content(ctx, path, outcome.content)?;
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
        ctx: &eframe::egui::Context,
        path: &Path,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        if let Some(idx) = self.open_document_index(path) {
            self.save_open_formatted_document(ctx, idx, content)
        } else {
            self.save_unopened_formatted_document(ctx, path, content)
        }
    }

    fn save_open_formatted_document(
        &mut self,
        ctx: &eframe::egui::Context,
        idx: usize,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        let path = self.state.document.open_documents[idx].path.clone();
        let is_active = self.state.document.active_doc_idx == Some(idx);
        let before = self.state.document.open_documents[idx].buffer.clone();
        {
            let doc = &mut self.state.document.open_documents[idx];
            doc.buffer = content.clone();
            doc.is_dirty = true;
            self.fs
                .save_document(doc)
                .map_err(|err| MarkdownFormatFailure::new(&path, err.to_string()))?;
        }
        self.record_formatted_undo(ctx, &path, &before, &content);
        self.refresh_after_format(path, &content, is_active);
        Ok(())
    }

    fn save_unopened_formatted_document(
        &mut self,
        ctx: &eframe::egui::Context,
        path: &Path,
        content: String,
    ) -> Result<(), MarkdownFormatFailure> {
        let before = self.load_format_source(path)?;
        let mut doc = katana_core::document::Document::new(path, content.clone());
        self.fs
            .save_document(&mut doc)
            .map_err(|err| MarkdownFormatFailure::new(path, err.to_string()))?;
        self.record_formatted_undo(ctx, path, &before, &content);
        self.refresh_formatted_diagnostics(path.to_path_buf(), &content);
        Ok(())
    }

    fn record_formatted_undo(
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

    fn show_markdown_format_summary(&mut self, summary: MarkdownFormattingSummary) {
        self.state.layout.status_message = Some((summary.status_message(), summary.status_type()));
    }
}
