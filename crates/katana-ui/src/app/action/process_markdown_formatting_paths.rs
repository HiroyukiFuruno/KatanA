use std::path::{Path, PathBuf};

use crate::markdown_formatting_bridge::MarkdownFormatFailure;
use crate::shell::KatanaApp;

pub(super) trait MarkdownFormattingPathOps {
    fn collect_workspace_markdown_paths(&self, root: &Path) -> Vec<PathBuf>;
    fn open_document_index(&self, path: &Path) -> Option<usize>;
    fn is_markdown_path(path: &Path) -> bool;
}

impl MarkdownFormattingPathOps for KatanaApp {
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
                !is_inside_ignored_directory(
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
}

pub(super) struct MarkdownFormattingPathFailureOps;

impl MarkdownFormattingPathFailureOps {
    pub(super) fn markdown_path_failure(path: &Path) -> MarkdownFormatFailure {
        let message = crate::i18n::I18nOps::get()
            .status
            .format_markdown_not_markdown
            .clone();
        MarkdownFormatFailure::new(path, message)
    }
}

fn is_inside_ignored_directory(root: &Path, path: &Path, ignored_directories: &[String]) -> bool {
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
