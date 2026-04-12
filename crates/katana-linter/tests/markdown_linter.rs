//! Parity regression tests for user-facing markdown diagnostics.
//!
//! These tests verify that each official markdownlint rule's
//! violation and valid (no false positive) cases behave correctly,
//! serving as the parity contract fixture corpus.

use katana_linter::markdown::*;
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
    // official_meta must be present for user-facing display
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

trait BoolExt {
    fn not(self) -> bool;
}
impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}
