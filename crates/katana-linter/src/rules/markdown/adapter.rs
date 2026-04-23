use std::path::Path;

/* WHY: Section: Adapter trait for future vendor linter integration
=======================================================
  When `katana-markdown-linter` (the independent crate) is ready,
  KatanA will switch from the internal engine to the vendor crate.
  This trait defines the stable contract between the two. */

/// A single lint diagnostic produced by a Markdown linter engine.
///
/// This is the vendor-neutral intermediate representation.
/// Both the internal engine and the future `katana-markdown-linter`
/// produce values of this type via their respective adapters.
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub rule_id: String,
    pub rule_name: String,
    pub message: String,
    pub severity: super::DiagnosticSeverity,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub fix: Option<LintFix>,
}

/// Auto-fix information attached to a diagnostic.
#[derive(Debug, Clone)]
pub struct LintFix {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub replacement: String,
}

/// Adapter trait that converts vendor-specific lint results into
/// KatanA's internal `MarkdownDiagnostic`.
///
/// # Design
///
/// - `lint`: returns diagnostics with optional per-diagnostic fix info
/// - `fix_all`: the **before → after** interface — takes content, returns
///   fully fixed content. No coordinate translation on the UI side.
///
/// # Implementors
///
/// - `InternalAdapter`: wraps the current built-in rule engine
/// - (future) `VendorAdapter`: wraps `katana-markdown-linter` crate
pub trait MarkdownLintAdapter {
    /// Run lint on the given content and return diagnostics.
    fn lint(
        &self,
        file_path: &Path,
        content: &str,
        config: &super::config::MarkdownLintConfig,
    ) -> Vec<LintDiagnostic>;

    /// Apply all available auto-fixes and return the corrected content.
    ///
    /// Returns `None` if no fixes are applicable (content unchanged).
    fn fix_all(
        &self,
        file_path: &Path,
        content: &str,
        config: &super::config::MarkdownLintConfig,
    ) -> Option<String>;
}
