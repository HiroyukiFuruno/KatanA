use katana_linter::markdown::MarkdownDiagnostic;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DiagnosticsState {
    pub problems: HashMap<PathBuf, Vec<MarkdownDiagnostic>>,
    pub is_panel_open: bool,
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            problems: HashMap::new(),
            is_panel_open: false,
        }
    }

    pub fn update_diagnostics(&mut self, path: PathBuf, diagnostics: Vec<MarkdownDiagnostic>) {
        if diagnostics.is_empty() {
            self.problems.remove(&path);
        } else {
            self.problems.insert(path, diagnostics);
        }
    }

    pub fn total_problems(&self) -> usize {
        self.problems.values().map(|v| v.len()).sum()
    }
}
