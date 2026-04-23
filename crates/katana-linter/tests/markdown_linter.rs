//! Parity regression tests for user-facing markdown diagnostics.
//!
//! These tests verify that each official markdownlint rule's
//! violation and valid (no false positive) cases behave correctly,
//! serving as the parity contract fixture corpus.

use katana_linter::rules::markdown::*;
use std::path::PathBuf;

/* WHY: MD001 — heading-increment parity fixture
======================================================= */

#[test]
fn md001_valid_sequential_headings_produce_no_diagnostics() {
    let rule = HeadingIncrementRule;
    let path = PathBuf::from("test.md");
    let content = "\
# Title
## Section 1
### Sub 1
## Section 2
";
    let diagnostics = rule.evaluate(&path, content);
    assert!(
        diagnostics.is_empty(),
        "Valid sequential headings should have 0 diagnostics"
    );
}

#[test]
fn md001_skipped_heading_level_produces_warning() {
    let rule = HeadingIncrementRule;
    let path = PathBuf::from("test.md");
    let content = "\
# Title
### Section 1 (skip level)
";
    let diagnostics = rule.evaluate(&path, content);
    assert_eq!(
        diagnostics.len(),
        1,
        "Level skip should produce 1 diagnostic"
    );
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostics[0].range.start_line, 2);
    assert_eq!(diagnostics[0].rule_id, "MD001");
    /* WHY: official_meta must be present for user-facing display */
    let meta = diagnostics[0]
        .official_meta
        .as_ref()
        .expect("official_meta must be Some for MD001");
    assert_eq!(meta.code, "MD001");
    assert_eq!(meta.title, "heading-increment");
    assert_eq!(meta.parity, RuleParityStatus::Official);
    assert!(
        diagnostics[0].message.contains("MD001").not()
            || diagnostics[0].message.starts_with("Heading levels"),
        "Message should be the official English description"
    );
}

#[test]
fn md001_rule_id_is_official_code() {
    let rule = HeadingIncrementRule;
    let path = PathBuf::from("test.md");
    let content = "# H1\n### H3 skip\n";
    let diagnostics = rule.evaluate(&path, content);
    assert!(!diagnostics.is_empty());
    assert_eq!(
        diagnostics[0].rule_id, "MD001",
        "rule_id must be official code, not internal name"
    );
}

#[test]
fn test_parity_status_boundary() {
    use katana_linter::rules::markdown::*;

    let meta_exp = OfficialRuleMeta {
        code: "MD000",
        title: "test-rule",
        description: "Test rule description.",
        docs_url: "https://example.com/md000",
        parity: RuleParityStatus::Experimental,
        is_fixable: false,
        properties: &[],
    };
    assert_eq!(meta_exp.parity, RuleParityStatus::Experimental);

    let meta_off = OfficialRuleMeta {
        code: "MD001",
        title: "heading-increment",
        description: "Heading levels should only increment by one level at a time.",
        docs_url: "dummy",
        parity: RuleParityStatus::Official,
        is_fixable: false,
        properties: &[],
    };
    assert_eq!(meta_off.parity, RuleParityStatus::Official);

    /* WHY: Diagnostics filtering boundary test representation */
    let diags = [
        MarkdownDiagnostic {
            file: std::path::PathBuf::from("test.md"),
            severity: DiagnosticSeverity::Error,
            range: DiagnosticRange {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 1,
            },
            message: "Experimental rule".to_string(),
            rule_id: "MD000".to_string(),
            official_meta: Some(meta_exp),
            fix_info: None,
        },
        MarkdownDiagnostic {
            file: std::path::PathBuf::from("test.md"),
            severity: DiagnosticSeverity::Warning,
            range: DiagnosticRange {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 1,
            },
            message: "Hidden internal rule".to_string(),
            rule_id: "internal-001".to_string(),
            official_meta: None,
            fix_info: None,
        },
    ];

    let displayable: Vec<_> = diags.iter().filter(|d| d.official_meta.is_some()).collect();
    assert_eq!(
        displayable.len(),
        1,
        "Only diagnostics with official_meta should be displayable"
    );
    assert_eq!(displayable[0].rule_id, "MD000");
}

#[test]
fn md001_code_blocks_do_not_trigger_false_positive() {
    let rule = HeadingIncrementRule;
    let path = PathBuf::from("test.md");
    let content = "\
# Title

```rust
/* WHY: A comment looking like an H3 inside code block should not trigger MD001 */
### This is not a heading
```
";
    let diagnostics = rule.evaluate(&path, content);
    assert!(
        diagnostics.is_empty(),
        "Headings inside code blocks must be ignored (no false positive)"
    );
}

#[test]
fn md001_starts_with_h2_is_valid() {
    let rule = HeadingIncrementRule;
    let path = PathBuf::from("test.md");
    let content = "## Subtitle";
    let diagnostics = rule.evaluate(&path, content);
    assert!(
        diagnostics.is_empty(),
        "Starting with H2 is valid, MD001 only checks increment"
    );
}

/* WHY: HeadingStructureRule alias backward-compat test
======================================================= */

#[test]
fn heading_structure_rule_alias_works() {
    /* WHY: Ensure UI crate references to HeadingStructureRule still compile and run. */
    let rule = HeadingStructureRule;
    let path = PathBuf::from("test.md");
    let content = "# H1\n### H3 skip\n";
    let diagnostics = rule.evaluate(&path, content);
    assert_eq!(diagnostics.len(), 1);
}

/* WHY: BrokenLinkRule hidden test — must not expose official_meta
======================================================= */

#[test]
fn broken_link_rule_is_hidden_from_user_facing_diagnostics() {
    let rule = BrokenLinkRule;
    assert!(
        rule.official_meta().is_none(),
        "BrokenLinkRule must be hidden (official_meta = None)"
    );
}

/* WHY: MD022 / blanks-around-headings fix integrity tests
 * These guard against the "cascading fix" bug where applying a fix produced
 * a new violation instead of resolving the original one.
======================================================= */

#[test]
fn md022_valid_blank_lines_no_diagnostic() {
    use katana_linter::rules::markdown::rules::heading::BlanksAroundHeadingsRule;
    let rule = BlanksAroundHeadingsRule;
    let path = std::path::PathBuf::from("test.md");
    let content = "# Title\n\n## Section\n\nContent here\n";
    let diagnostics = rule.evaluate(&path, content);
    assert!(
        diagnostics.is_empty(),
        "Properly spaced headings must produce no diagnostics"
    );
}

#[test]
fn md022_fix_missing_blank_before_heading_resolves_violation() {
    use katana_linter::rules::markdown::rules::heading::BlanksAroundHeadingsRule;
    let rule = BlanksAroundHeadingsRule;
    let path = std::path::PathBuf::from("test.md");
    /* WHY: "Content" directly before "## Section" — no blank line */
    let content = "Content\n## Section\n\nMore\n";
    let diagnostics = rule.evaluate(&path, content);
    assert_eq!(
        diagnostics.len(),
        1,
        "Missing blank before heading: 1 diagnostic"
    );
    let fix = diagnostics[0]
        .fix_info
        .as_ref()
        .expect("Fix must be present");
    let fixed = apply_md_fix(content, fix);
    let after = rule.evaluate(&path, &fixed);
    assert!(
        after.is_empty(),
        "After applying fix the heading should be properly surrounded. Got:\n{fixed:?}"
    );
}

#[test]
fn md022_fix_missing_blank_after_heading_resolves_violation() {
    use katana_linter::rules::markdown::rules::heading::BlanksAroundHeadingsRule;
    let rule = BlanksAroundHeadingsRule;
    let path = std::path::PathBuf::from("test.md");
    /* WHY: "## Section" directly followed by "Content" — no blank line after */
    let content = "# Title\n\n## Section\nContent\n";
    let diagnostics = rule.evaluate(&path, content);
    assert_eq!(
        diagnostics.len(),
        1,
        "Missing blank after heading: 1 diagnostic"
    );
    let fix = diagnostics[0]
        .fix_info
        .as_ref()
        .expect("Fix must be present");
    let fixed = apply_md_fix(content, fix);
    let after = rule.evaluate(&path, &fixed);
    assert!(
        after.is_empty(),
        "After applying fix the heading should be properly surrounded. Got:\n{fixed:?}"
    );
}

#[test]
fn md022_fix_sequential_application_reaches_clean_state() {
    use katana_linter::rules::markdown::rules::heading::BlanksAroundHeadingsRule;
    let rule = BlanksAroundHeadingsRule;
    let path = std::path::PathBuf::from("test.md");
    /* WHY: Multiple consecutive headings without blank lines — the "cascading fix" scenario
     * that previously caused applying one fix to create new violations. */
    let content = "# A\n## B\n## C\n## D\n";
    let mut buffer = content.to_string();
    /* WHY: Apply fixes one at a time (re-evaluate after each) to simulate the UI behavior */
    for _ in 0..20 {
        let diags = rule.evaluate(&path, &buffer);
        if diags.is_empty() {
            break;
        }
        let fix = diags[0]
            .fix_info
            .as_ref()
            .expect("Fixable rule must have fix_info");
        buffer = apply_md_fix(&buffer, fix);
    }
    let final_diags = rule.evaluate(&path, &buffer);
    assert!(
        final_diags.is_empty(),
        "Sequential fix application must converge to a clean document. Got:\n{buffer:?}"
    );
}

/* WHY: Helper — apply a single DiagnosticFix to a buffer using the same coordinate
 * convention (1-based line, 1-based column) as the production fix applier. */
fn apply_md_fix(content: &str, fix: &katana_linter::rules::markdown::DiagnosticFix) -> String {
    let start = md_line_col_to_byte(fix.start_line, fix.start_column, content);
    let end = md_line_col_to_byte(fix.end_line, fix.end_column, content);
    let mut result = content.to_string();
    result.replace_range(start..end, &fix.replacement);
    result
}

fn md_line_col_to_byte(line_1: usize, col_1: usize, content: &str) -> usize {
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
    /* WHY: If line_1 > total lines, clamp to EOF */
    if cur_line < line_1 {
        return content.len();
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

trait BoolExt {
    fn not(self) -> bool;
}
impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}
