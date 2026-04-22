use std::path::PathBuf;

/* WHY: Section: Official markdownlint metadata catalog
=======================================================
 Defines the official rule metadata used in user-facing diagnostics.
 Internal rule names are hidden from user-facing output; only official
 markdownlint codes and descriptions are surfaced. */

/// Parity status of a rule relative to official markdownlint behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleParityStatus {
    /// Rule behavior is aligned with official markdownlint specification.
    Official,
    /// Rule is implemented but parity is not yet fully verified; shown with visual distinction.
    Experimental,
    /// Rule is internal-only and must not appear in user-facing diagnostics.
    Hidden,
}

/// Canonical metadata for an official markdownlint rule.
#[derive(Debug, Clone)]
pub struct OfficialRuleMeta {
    /// Official rule code, e.g. "MD001".
    pub code: &'static str,
    /// Official rule title (short name), e.g. "heading-increment".
    pub title: &'static str,
    /// English description shown in Problems Panel.
    pub description: &'static str,
    /// Official documentation URL at markdownlint GitHub.
    pub docs_url: &'static str,
    /// Parity status of this rule.
    pub parity: RuleParityStatus,
    /// Indicates if this rule can be automatically fixed.
    pub is_fixable: bool,
}

/* WHY: Section: Diagnostic types
======================================================= */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticFix {
    /// 1-indexed line number where the fix starts
    pub start_line: usize,
    /// 1-indexed column number where the fix starts
    pub start_column: usize,
    /// 1-indexed line number where the fix ends
    pub end_line: usize,
    /// 1-indexed column number where the fix ends
    pub end_column: usize,
    /// The replacement string
    pub replacement: String,
}

/// A single diagnostic item produced by a markdown linting rule.
///
/// `official_meta` is `Some` for rules with `RuleParityStatus::Official` or
/// `RuleParityStatus::Experimental`, and `None` for hidden internal rules.
/// The UI layer must only surface diagnostics that have `official_meta`.
#[derive(Debug, Clone)]
pub struct MarkdownDiagnostic {
    pub file: PathBuf,
    pub severity: DiagnosticSeverity,
    pub range: DiagnosticRange,
    /// English message derived from `OfficialRuleMeta::description` for official rules.
    pub message: String,
    /// Official rule code (e.g. "MD001") for official/experimental rules;
    /// internal rule id for hidden rules.
    pub rule_id: String,
    /// Official markdownlint metadata; `None` for hidden internal rules.
    pub official_meta: Option<OfficialRuleMeta>,
    /// Optional fix information for auto-fixable rules.
    pub fix_info: Option<DiagnosticFix>,
}

impl std::fmt::Display for MarkdownDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            DiagnosticSeverity::Error => "ERROR",
            DiagnosticSeverity::Warning => "WARN",
            DiagnosticSeverity::Info => "INFO",
        };
        write!(
            f,
            "[{}] {} {}:{}:{} — {}",
            sev,
            self.rule_id,
            self.file.display(),
            self.range.start_line,
            self.range.start_column,
            self.message
        )
    }
}
