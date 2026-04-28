use super::{
    DiffCell, DiffLine, DiffLineKind, DiffModelOps, InlineDiffRow, SplitDiffRow, TextRange,
};

#[test]
fn build_detects_removed_and_added_lines() {
    let model = DiffModelOps::build("alpha\nbeta\ngamma\n", "alpha\nbravo\ngamma\n");

    assert_eq!(model.inline_rows.len(), 4);
    assert!(
        matches!(&model.inline_rows[0], InlineDiffRow::Collapsed(block) if block.line_count == 1)
    );
    assert_eq!(
        model.inline_rows[1],
        InlineDiffRow::Line(line_with_highlight(
            DiffLine::removed(2, "beta"),
            vec![TextRange::new(1, 3)]
        ))
    );
    assert_eq!(
        model.inline_rows[2],
        InlineDiffRow::Line(line_with_highlight(
            DiffLine::added(2, "bravo"),
            vec![TextRange::new(1, 2), TextRange::new(3, 5)]
        ))
    );
    assert!(
        matches!(&model.inline_rows[3], InlineDiffRow::Collapsed(block) if block.line_count == 2)
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
                highlight_ranges: vec![TextRange::new(1, 3)],
            }),
            after: Some(DiffCell {
                line_number: 2,
                text: "bravo".to_string(),
                kind: DiffLineKind::Added,
                highlight_ranges: vec![TextRange::new(1, 2), TextRange::new(3, 5)],
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
                highlight_ranges: vec![TextRange::new(0, 1)],
            }),
            after: Some(DiffCell {
                line_number: 2,
                text: "x".to_string(),
                kind: DiffLineKind::Added,
                highlight_ranges: vec![TextRange::new(0, 1)],
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
                highlight_ranges: vec![TextRange::new(0, 1)],
            }),
        })
    );
}

#[test]
fn build_highlights_only_changed_characters_in_replaced_lines() {
    let before_line = "**Verification Targets:**";
    let after_line = "# Verification Targets:";
    let model = DiffModelOps::build(before_line, after_line);
    let before_len = before_line.chars().count();

    let SplitDiffRow::Line(line) = &model.split_rows[0] else {
        panic!("expected replaced split row");
    };

    assert_eq!(
        line.before.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&vec![
            TextRange::new(0, 2),
            TextRange::new(before_len - 2, before_len)
        ])
    );
    assert_eq!(
        line.after.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&vec![TextRange::new(0, 2)])
    );
}

#[test]
fn build_highlights_removed_space_in_replaced_lines() {
    let model = DiffModelOps::build("a b", "ab");

    let SplitDiffRow::Line(line) = &model.split_rows[0] else {
        panic!("expected replaced split row");
    };

    assert_eq!(
        line.before.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&vec![TextRange::new(1, 2)])
    );
    assert_eq!(
        line.after.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&Vec::<TextRange>::new())
    );
}

#[test]
fn build_highlights_added_space_in_replaced_lines() {
    let model = DiffModelOps::build("ab", "a b");

    let SplitDiffRow::Line(line) = &model.split_rows[0] else {
        panic!("expected replaced split row");
    };

    assert_eq!(
        line.before.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&Vec::<TextRange>::new())
    );
    assert_eq!(
        line.after.as_ref().map(|cell| &cell.highlight_ranges),
        Some(&vec![TextRange::new(1, 2)])
    );
}

#[test]
fn build_preserves_added_trailing_newline_as_empty_line() {
    let model = DiffModelOps::build("alpha", "alpha\n");

    let SplitDiffRow::Line(line) = &model.split_rows[1] else {
        panic!("expected added trailing empty line");
    };

    assert_eq!(line.before, None);
    assert_eq!(
        line.after,
        Some(DiffCell {
            line_number: 2,
            text: String::new(),
            kind: DiffLineKind::Added,
            highlight_ranges: Vec::new(),
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
    assert_eq!(block.line_count, 10);
    assert_eq!(block.lines.len(), 10);
    assert_eq!(
        model.inline_rows[1],
        InlineDiffRow::Line(line_with_highlight(
            DiffLine::removed(11, "old"),
            vec![TextRange::new(0, 3)]
        ))
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
    assert_eq!(compacted.len(), 1);
    let InlineDiffRow::Collapsed(block) = &compacted[0] else {
        panic!("expected collapsed block");
    };
    assert_eq!(block.line_count, 3);
    assert_eq!(block.lines, lines);
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

    let collapsed_idx = compacted
        .iter()
        .position(|r| matches!(r, InlineDiffRow::Collapsed(_)))
        .expect("expected a collapsed block");

    assert_eq!(compacted.len(), 3);
    assert_eq!(collapsed_idx, 1);

    /* WHY: trailing changed line preserved */
    assert!(
        matches!(compacted.last().unwrap(), InlineDiffRow::Line(l) if l.kind == DiffLineKind::Added)
    );
}

fn line_with_highlight(mut line: DiffLine, ranges: Vec<TextRange>) -> DiffLine {
    line.highlight_ranges = ranges;
    line
}
