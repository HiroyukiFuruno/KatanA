use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct LintFixBatch {
    pub path: PathBuf,
    pub fixes: Vec<katana_markdown_linter::rules::markdown::DiagnosticFix>,
}
