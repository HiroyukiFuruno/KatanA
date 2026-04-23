use katana_linter::rules::markdown::MarkdownDiagnostic;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DiagnosticsState {
    pub problems: BTreeMap<PathBuf, Vec<MarkdownDiagnostic>>,
    pub is_panel_open: bool,
    pub expand_all: Option<bool>,
    pub last_buffer_update: Option<std::time::Instant>,
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            problems: BTreeMap::new(),
            is_panel_open: false,
            expand_all: None,
            last_buffer_update: None,
        }
    }

    pub fn update_diagnostics(&mut self, path: PathBuf, diagnostics: Vec<MarkdownDiagnostic>) {
        if diagnostics.is_empty() {
            self.problems.remove(&path);
        } else {
            self.problems.insert(path, diagnostics);
        }
    }

    pub fn get_file_diagnostics(&self, path: &std::path::Path) -> &[MarkdownDiagnostic] {
        self.problems.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn total_problems(&self) -> usize {
        self.problems
            .values()
            .map(|v| v.iter().filter(|d| d.official_meta.is_some()).count())
            .sum()
    }

    /// Remove all diagnostics for a deleted file or directory prefix.
    /// Called when a file/dir is removed so Problems panel stays in sync.
    pub fn remove_file_diagnostics(&mut self, path: &std::path::Path) {
        /* WHY: For files, one exact key. For directories, remove all paths under it. */
        if path.is_file() || !path.exists() {
            self.problems.remove(path);
        } else {
            self.problems.retain(|k, _| !k.starts_with(path));
        }
    }
}
