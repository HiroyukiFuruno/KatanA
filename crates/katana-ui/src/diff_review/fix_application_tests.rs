use katana_markdown_linter::rules::markdown::DiagnosticFix;

use super::DiagnosticFixApplicationOps;

fn fix(start_column: usize, end_column: usize, replacement: &str) -> DiagnosticFix {
    DiagnosticFix {
        start_line: 1,
        start_column,
        end_line: 1,
        end_column,
        replacement: replacement.to_string(),
    }
}

#[test]
fn apply_builds_after_content_from_diagnostic_fixes() {
    let content = "alpha\n";
    let after = DiagnosticFixApplicationOps::apply_result(content, &[fix(1, 6, "beta")]).content;

    assert_eq!(after, "beta\n");
}

#[test]
fn apply_skips_overlapping_fixes_like_kml() {
    let content = "alpha\n";
    let after =
        DiagnosticFixApplicationOps::apply_result(content, &[fix(1, 3, "AL"), fix(2, 4, "LP")])
            .content;

    assert_eq!(after, "aLPha\n");
}

#[test]
fn apply_result_reports_skipped_overlapping_fixes() {
    let content = "alpha\n";
    let result =
        DiagnosticFixApplicationOps::apply_result(content, &[fix(1, 3, "AL"), fix(2, 4, "LP")]);

    assert_eq!(result.applied_fixes, 1);
    assert_eq!(result.details.len(), 2);
    assert_eq!(
        result.details.iter().filter(|detail| detail.applied).count(),
        1
    );
    assert_eq!(
        result.details.iter().filter(|detail| !detail.applied).count(),
        1
    );
}
