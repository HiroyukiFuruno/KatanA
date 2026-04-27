use super::{DiffCell, DiffLine, DiffLineKind, DiffModelOps, InlineDiffRow, SplitDiffRow};

#[test]
fn build_detects_removed_and_added_lines() {
    let model = DiffModelOps::build("alpha\nbeta\ngamma\n", "alpha\nbravo\ngamma\n");

    assert_eq!(
        model.inline_rows,
        vec![
            InlineDiffRow::Line(DiffLine::unchanged(1, 1, "alpha")),
            InlineDiffRow::Line(DiffLine::removed(2, "beta")),
            InlineDiffRow::Line(DiffLine::added(2, "bravo")),
            InlineDiffRow::Line(DiffLine::unchanged(3, 3, "gamma")),
        ]
    );
    assert_eq!(model.stats.added_count, 1);
    assert_eq!(model.stats.removed_count, 1);
    assert_eq!(
        model.split_rows[1],
        SplitDiffRow::Line(super::SplitDiffLine {
            before: Some(DiffCell {
                line_number: 2,
                text: "beta".to_string(),
                kind: DiffLineKind::Removed,
            }),
            after: Some(DiffCell {
                line_number: 2,
                text: "bravo".to_string(),
                kind: DiffLineKind::Added,
            }),
        })
    );
}

#[test]
fn build_pairs_multi_line_replacements_in_split_rows() {
    let model = DiffModelOps::build("a\nb\nc\n", "a\nx\ny\nc\n");

    assert_eq!(
        model.split_rows[1],
        SplitDiffRow::Line(super::SplitDiffLine {
            before: Some(DiffCell {
                line_number: 2,
                text: "b".to_string(),
                kind: DiffLineKind::Removed,
            }),
            after: Some(DiffCell {
                line_number: 2,
                text: "x".to_string(),
                kind: DiffLineKind::Added,
            }),
        })
    );
    assert_eq!(
        model.split_rows[2],
        SplitDiffRow::Line(super::SplitDiffLine {
            before: None,
            after: Some(DiffCell {
                line_number: 3,
                text: "y".to_string(),
                kind: DiffLineKind::Added,
            }),
        })
    );
}

#[test]
fn build_collapses_long_unchanged_runs_with_line_numbers() {
    let before = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\nold\n";
    let after = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\nnew\n";
    let model = DiffModelOps::build(before, after);

    let SplitDiffRow::Collapsed(block) = &model.split_rows[0] else {
        panic!("expected leading unchanged block");
    };

    assert_eq!(block.before_start_line_number, 1);
    assert_eq!(block.after_start_line_number, 1);
    assert_eq!(block.line_count, 7);
    assert_eq!(
        model.inline_rows[1],
        InlineDiffRow::Line(DiffLine::unchanged(8, 8, "8"))
    );
}
