use super::fix_preview_model::{FixPreviewModelOps, MAX_DIFF_ROWS};
use katana_markdown_linter::rules::markdown::DiagnosticFix;

fn fix(start_line: usize, end_line: usize, replacement: &str) -> DiagnosticFix {
    DiagnosticFix {
        start_line,
        start_column: 1,
        end_line,
        end_column: 1,
        replacement: replacement.to_string(),
    }
}

#[test]
fn build_collects_original_and_replacement_lines() {
    let content = "keep\nold first\nold second\nkeep";
    let rows = FixPreviewModelOps::build(&fix(2, 3, "new first\nnew second"), content)
        .expect("valid range should build preview rows");

    assert_eq!(rows.removed, vec!["old first", "old second"]);
    assert_eq!(rows.added, vec!["new first", "new second"]);
    assert!(!rows.removed_truncated);
    assert!(!rows.added_truncated);
}

#[test]
fn build_limits_long_diff_rows() {
    let content = (0..MAX_DIFF_ROWS + 2)
        .map(|line_number| format!("old {line_number}"))
        .collect::<Vec<_>>()
        .join("\n");
    let replacement = (0..MAX_DIFF_ROWS + 3)
        .map(|line_number| format!("new {line_number}"))
        .collect::<Vec<_>>()
        .join("\n");
    let rows = FixPreviewModelOps::build(&fix(1, MAX_DIFF_ROWS + 2, &replacement), &content)
        .expect("valid range should build preview rows");

    assert_eq!(rows.removed.len(), MAX_DIFF_ROWS);
    assert_eq!(rows.added.len(), MAX_DIFF_ROWS);
    assert!(rows.removed_truncated);
    assert!(rows.added_truncated);
}

#[test]
fn build_rejects_invalid_line_range() {
    assert!(FixPreviewModelOps::build(&fix(0, 1, "new"), "old").is_none());
    assert!(FixPreviewModelOps::build(&fix(2, 1, "new"), "old\nnext").is_none());
    assert!(FixPreviewModelOps::build(&fix(3, 3, "new"), "old\nnext").is_none());
}
