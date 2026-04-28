use katana_markdown_linter::rules::markdown::MarkdownDiagnostic;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DiagnosticsState {
    pub problems: BTreeMap<PathBuf, Vec<MarkdownDiagnostic>>,
    pub content_hashes: BTreeMap<PathBuf, u64>,
    pub content_snapshots: BTreeMap<PathBuf, String>,
    pub is_panel_open: bool,
    pub expand_all: Option<bool>,
    pub last_buffer_update: Option<std::time::Instant>,
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            problems: BTreeMap::new(),
            content_hashes: BTreeMap::new(),
            content_snapshots: BTreeMap::new(),
            is_panel_open: false,
            expand_all: None,
            last_buffer_update: None,
        }
    }

    pub fn is_current(&self, path: &std::path::Path, content: &str) -> bool {
        self.content_hashes
            .get(path)
            .is_some_and(|hash| *hash == Self::content_hash(content))
    }

    pub fn update_diagnostics_for_content(
        &mut self,
        path: PathBuf,
        content: &str,
        diagnostics: Vec<MarkdownDiagnostic>,
    ) {
        self.content_hashes
            .insert(path.clone(), Self::content_hash(content));
        self.content_snapshots.insert(path.clone(), content.to_string());
        self.update_diagnostics(path, diagnostics);
    }

    pub fn content_snapshot(&self, path: &std::path::Path) -> Option<&str> {
        self.content_snapshots.get(path).map(String::as_str)
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
            self.content_hashes.remove(path);
            self.content_snapshots.remove(path);
        } else {
            self.problems.retain(|k, _| !k.starts_with(path));
            self.content_hashes.retain(|k, _| !k.starts_with(path));
            self.content_snapshots.retain(|k, _| !k.starts_with(path));
        }
    }

    fn content_hash(content: &str) -> u64 {
        katana_core::document::DocumentOps::compute_hash(content)
    }
}
