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

#[test]
fn compact_inline_rows_short_and_long_runs() {
    use super::{DiffCompactionOps, DiffLine, InlineDiffRow};

    /* WHY: short run (no collapse) */
    let short_lines = vec![
        DiffLine::unchanged(1, 1, "1"),
        DiffLine::removed(2, "2"),
        DiffLine::added(2, "2b"),
        DiffLine::unchanged(3, 3, "3"),
    ];
    let compacted_short = DiffCompactionOps::compact_inline_rows(&short_lines);
    assert!(!compacted_short.is_empty());

    /* WHY: long run (should collapse) */
    let mut long_lines = Vec::new();
    for i in 1..15 {
        long_lines.push(DiffLine::unchanged(i, i, &format!("{}", i)));
    }
    let compacted_long = DiffCompactionOps::compact_inline_rows(&long_lines);
    assert!(
        compacted_long
            .iter()
            .any(|r| matches!(r, InlineDiffRow::Collapsed(_)))
    );
}

#[test]
fn compact_inline_rows_all_unchanged_short_run() {
    use super::{DiffCompactionOps, DiffLine, InlineDiffRow};

    let mut lines = Vec::new();
    for i in 1..=3 {
        lines.push(DiffLine::unchanged(i, i, &format!("{}", i)));
    }
    let compacted = DiffCompactionOps::compact_inline_rows(&lines);
    assert_eq!(compacted.len(), 3);
    assert!(matches!(compacted[0], InlineDiffRow::Line(_)));
}

#[test]
fn build_handles_trailing_removed_lines() {
    let model = DiffModelOps::build("a\nb\nc\nd\n", "a\nb\n");
    assert!(
        model
            .inline_rows
            .iter()
            .any(|r| matches!(r, InlineDiffRow::Line(l) if l.kind == DiffLineKind::Removed))
    );
}

#[test]
fn compact_inline_rows_middle_long_run_keeps_context() {
    use super::{DiffCompactionOps, DiffLine, InlineDiffRow};

    let mut lines = Vec::new();
    lines.push(DiffLine::removed(1, "pre-change"));
    for i in 0..11 {
        lines.push(DiffLine::unchanged(2 + i, 2 + i, &format!("L{}", i + 1)));
    }
    lines.push(DiffLine::added(200, "post-change"));

    let compacted = DiffCompactionOps::compact_inline_rows(&lines);

    /* WHY: leading changed line preserved */
    assert!(
        matches!(compacted.first().unwrap(), InlineDiffRow::Line(l) if l.kind == DiffLineKind::Removed)
    );

    /* WHY: collapsed block exists */
    let collapsed_idx = compacted
        .iter()
        .position(|r| matches!(r, InlineDiffRow::Collapsed(_)))
        .expect("expected a collapsed block");

    /* WHY: ensure front context lines (3) are present immediately before collapsed */
    for i in 1..=3 {
        assert!(matches!(
            compacted[collapsed_idx - i],
            InlineDiffRow::Line(_)
        ));
    }

    /* WHY: ensure back context lines (3) are present immediately after collapsed */
    for i in 1..=3 {
        assert!(matches!(
            compacted[collapsed_idx + i],
            InlineDiffRow::Line(_)
        ));
    }

    /* WHY: trailing changed line preserved */
    assert!(
        matches!(compacted.last().unwrap(), InlineDiffRow::Line(l) if l.kind == DiffLineKind::Added)
    );
}
