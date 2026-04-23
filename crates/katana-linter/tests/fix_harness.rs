//! # Lint Fix Test Harness
//!
//! A portable, self-contained test harness for verifying `DiagnosticFix` correctness.
//!
//! ## Design Contract
//!
//! This module is intentionally designed for **repository portability**.
//! It has zero dependencies beyond `katana_linter` itself — no Katana UI, no
//! platform crates. This allows the entire harness to be extracted into an
//! independent integration test crate or a future `katana-linter` repository.
//!
//! ## Interface
//!
//! | Type / fn                   | Role                                                 |
//! |----------------------------|------------------------------------------------------|
//! | `LintFixScenario`           | Declarative test case (input → expected output)      |
//! | `LintFixScenario::run()`    | Execute: evaluate → apply fix → re-evaluate          |
//! | `LintFixResult`             | Outcome: clean / residual diagnostics                |
//! | `BulkFixScenario`           | Multi-fix variant (fix_all path)                     |
//! | `BulkFixScenario::run()`    | Apply all fixes in one pass (descending sort)        |
//! | `apply_single_fix()`        | Low-level: apply one `DiagnosticFix` to a buffer    |
//! | `apply_bulk_fixes()`        | Low-level: apply sorted list of fixes to a buffer   |
//! | `byte_offset()`             | Coordinate converter (1-based line/col → byte index)|

use katana_linter::rules::markdown::{DiagnosticFix, MarkdownDiagnostic, MarkdownRule};

// ─────────────────────────────────────────────────────────
// Core coordinate utility
// ─────────────────────────────────────────────────────────

/// Convert 1-based (line, col) coordinates to a byte offset within `content`.
///
/// This mirrors the production logic in `document_edit.rs` / `EditorLogicOps`.
/// It is intentionally duplicated here so the harness remains self-contained.
pub fn byte_offset(line_1: usize, col_1: usize, content: &str) -> usize {
    let mut cur_line = 1usize;
    let mut line_start = 0usize;
    for (byte_idx, c) in content.char_indices() {
        if cur_line == line_1 {
            line_start = byte_idx;
            break;
        }
        if c == '\n' {
            cur_line += 1;
        }
    }
    if cur_line < line_1 {
        return content.len(); // clamp to EOF
    }
    let col0 = col_1.saturating_sub(1);
    let mut byte_idx = line_start;
    for (col, (off, c)) in content[line_start..].char_indices().enumerate() {
        if col == col0 || c == '\n' {
            return line_start + off;
        }
        byte_idx = line_start + off + c.len_utf8();
    }
    byte_idx
}

// ─────────────────────────────────────────────────────────
// Low-level fix appliers
// ─────────────────────────────────────────────────────────

/// Apply a single `DiagnosticFix` to `content` and return the mutated string.
pub fn apply_single_fix(content: &str, fix: &DiagnosticFix) -> String {
    let start = byte_offset(fix.start_line, fix.start_column, content);
    let end = byte_offset(fix.end_line, fix.end_column, content);
    let mut result = content.to_string();
    result.replace_range(start..end, &fix.replacement);
    result
}

/// Apply a list of fixes to `content` in a single pass.
///
/// Fixes are sorted **descending** by (start_line, start_column) so that
/// earlier replacements do not invalidate later byte offsets — mirroring
/// the production `handle_apply_lint_fixes` sort in `document_edit.rs`.
pub fn apply_bulk_fixes(content: &str, fixes: &[DiagnosticFix]) -> String {
    let mut sorted = fixes.to_vec();
    sorted.sort_by(|a, b| {
        b.start_line
            .cmp(&a.start_line)
            .then_with(|| b.start_column.cmp(&a.start_column))
    });

    let mut result = content.to_string();
    for fix in &sorted {
        let start = byte_offset(fix.start_line, fix.start_column, &result);
        let end = byte_offset(fix.end_line, fix.end_column, &result);
        if start <= end && end <= result.len() {
            result.replace_range(start..end, &fix.replacement);
        }
    }
    result
}

// ─────────────────────────────────────────────────────────
// Single-fix scenario
// ─────────────────────────────────────────────────────────

/// The outcome of running a `LintFixScenario`.
#[derive(Debug)]
pub struct LintFixResult {
    /// The document content after the fix was applied.
    pub fixed_content: String,
    /// Diagnostics remaining after the fix — empty means clean state.
    pub residual_diagnostics: Vec<MarkdownDiagnostic>,
}

impl LintFixResult {
    /// Assert that no diagnostics remain after fix application.
    #[track_caller]
    pub fn assert_clean(&self) {
        assert!(
            self.residual_diagnostics.is_empty(),
            "Expected clean document after fix, but {} diagnostic(s) remain:\n\
             fixed content:\n{:?}\nresidual: {:#?}",
            self.residual_diagnostics.len(),
            self.fixed_content,
            self.residual_diagnostics,
        );
    }

    /// Assert that exactly `n` diagnostics remain after fix application.
    ///
    /// Used in scenarios where partial resolution is expected (e.g. bulk-fix
    /// path that does not converge in a single pass — see FB29).
    #[allow(dead_code)] /* WHY: API surface for future bulk-fix residual assertions */
    #[track_caller]
    pub fn assert_residual_count(&self, n: usize) {
        assert_eq!(
            self.residual_diagnostics.len(),
            n,
            "Expected {n} residual diagnostic(s), got {}.\nfixed content:\n{:?}",
            self.residual_diagnostics.len(),
            self.fixed_content,
        );
    }
}

/// A declarative single-fix test scenario.
///
/// ```rust
/// use fix_harness::LintFixScenario;
/// use katana_linter::rules::markdown::rules::heading::BlanksAroundHeadingsRule;
///
/// LintFixScenario {
///     rule: &BlanksAroundHeadingsRule,
///     description: "missing blank line before heading",
///     input: "Content\n## Section\n\nMore\n",
///     expected_initial_count: 1,
/// }
/// .run()
/// .assert_clean();
/// ```
pub struct LintFixScenario<'a> {
    /// The rule under test — must implement `MarkdownRuleOps`.
    pub rule: &'a dyn MarkdownRule,
    /// Human-readable description shown in assertion failures.
    pub description: &'static str,
    /// The input Markdown content that should trigger a fixable diagnostic.
    pub input: &'static str,
    /// Number of diagnostics expected on the initial evaluation (pre-fix).
    pub expected_initial_count: usize,
}

impl LintFixScenario<'_> {
    /// Run the scenario: evaluate → pick first fix → apply → re-evaluate.
    ///
    /// # Panics
    /// Panics if `expected_initial_count` doesn't match or no `fix_info` is present.
    #[track_caller]
    pub fn run(&self) -> LintFixResult {
        let path = std::path::PathBuf::from("test.md");
        let initial = self.rule.evaluate(&path, self.input);

        assert_eq!(
            initial.len(),
            self.expected_initial_count,
            "[{}] Expected {} initial diagnostic(s), got {}",
            self.description,
            self.expected_initial_count,
            initial.len(),
        );

        let fix = initial[0]
            .fix_info
            .as_ref()
            .unwrap_or_else(|| panic!("[{}] diagnostic[0] must have fix_info", self.description));

        let fixed_content = apply_single_fix(self.input, fix);
        let residual_diagnostics = self.rule.evaluate(&path, &fixed_content);

        LintFixResult {
            fixed_content,
            residual_diagnostics,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Bulk-fix scenario
// ─────────────────────────────────────────────────────────

/// The outcome of running a `BulkFixScenario`.
#[derive(Debug)]
pub struct BulkFixResult {
    /// The document content after all fixes were applied in one pass.
    pub fixed_content: String,
    /// Diagnostics remaining after bulk application.
    pub residual_diagnostics: Vec<MarkdownDiagnostic>,
}

impl BulkFixResult {
    /// Assert that no diagnostics remain after bulk fix application.
    #[track_caller]
    pub fn assert_clean(&self) {
        assert!(
            self.residual_diagnostics.is_empty(),
            "Expected clean document after bulk fix, but {} diagnostic(s) remain:\n\
             fixed content:\n{:?}\nresidual: {:#?}",
            self.residual_diagnostics.len(),
            self.fixed_content,
            self.residual_diagnostics,
        );
    }
}

/// A declarative bulk-fix test scenario (fix_all path).
///
/// Collects all fixable diagnostics from one evaluation pass and applies
/// them all at once — mirrors the "Fix All" UI button behaviour.
pub struct BulkFixScenario<'a> {
    /// The rule under test.
    pub rule: &'a dyn MarkdownRule,
    /// Human-readable description shown in assertion failures.
    pub description: &'static str,
    /// The input Markdown content that should trigger one or more fixable diagnostics.
    pub input: &'static str,
    /// Minimum number of initial diagnostics (all must be fixable).
    pub expected_min_initial_count: usize,
}

impl BulkFixScenario<'_> {
    /// Run bulk-fix: evaluate → collect all fix_infos → apply all → re-evaluate.
    #[track_caller]
    pub fn run(&self) -> BulkFixResult {
        let path = std::path::PathBuf::from("test.md");
        let initial = self.rule.evaluate(&path, self.input);

        assert!(
            initial.len() >= self.expected_min_initial_count,
            "[{}] Expected at least {} initial diagnostic(s), got {}",
            self.description,
            self.expected_min_initial_count,
            initial.len(),
        );

        let fixes: Vec<DiagnosticFix> = initial.iter().filter_map(|d| d.fix_info.clone()).collect();

        assert!(
            !fixes.is_empty(),
            "[{}] No fixable diagnostics found — cannot run bulk scenario",
            self.description,
        );

        let fixed_content = apply_bulk_fixes(self.input, &fixes);
        let residual_diagnostics = self.rule.evaluate(&path, &fixed_content);

        BulkFixResult {
            fixed_content,
            residual_diagnostics,
        }
    }
}
