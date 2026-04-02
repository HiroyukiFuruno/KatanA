use katana_linter::markdown::*;
use std::path::PathBuf;

#[test]
fn test_heading_structure_rule() {
    let rule = HeadingStructureRule;
    let path = PathBuf::from("test.md");

    let content_ok = "\
# Title
## Section 1
### Sub 1
## Section 2
";
    let diagnostics = rule.evaluate(&path, content_ok);
    assert!(
        diagnostics.is_empty(),
        "Valid content should have 0 diagnostics"
    );

    let content_bad = "\
# Title
### Section 1 (skip level)
";
    let diagnostics = rule.evaluate(&path, content_bad);
    assert_eq!(
        diagnostics.len(),
        1,
        "Level skip should produce 1 diagnostic"
    );
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostics[0].range.start_line, 2);
    assert_eq!(
        diagnostics[0].message,
        "Heading level skipped from h1 to h3"
    );
}
