use super::split_model::DiffSplitModelOps;
use super::split_pairing::DiffSplitPairingOps;
use super::{
    DiffCell, DiffLine, DiffLineKind, DiffModelOps, InlineDiffRow, SplitDiffLine, SplitDiffRow,
    TextRange,
};

#[test]
fn build_empty_documents_has_no_diff_rows() {
    let model = DiffModelOps::build("", "");

    assert!(model.inline_rows.is_empty());
    assert!(model.split_rows.is_empty());
    assert_eq!(model.stats.added_count, 0);
    assert_eq!(model.stats.removed_count, 0);
}

#[test]
fn build_all_removed_document_uses_trailing_removed_lines() {
    let model = DiffModelOps::build("stale", "");

    assert_eq!(model.stats.removed_count, 1);
    assert_eq!(
        model.split_rows,
        vec![SplitDiffRow::Line(SplitDiffLine {
            before: Some(DiffCell {
                line_number: 1,
                text: "stale".to_string(),
                kind: DiffLineKind::Removed,
                highlight_ranges: vec![TextRange::new(0, 5)],
            }),
            after: None,
        })]
    );
}

#[test]
fn split_model_flushes_changes_before_plain_unchanged_line() {
    let rows = vec![
        InlineDiffRow::Line(DiffLine::removed(1, "old")),
        InlineDiffRow::Line(DiffLine::added(1, "new")),
        InlineDiffRow::Line(DiffLine::unchanged(2, 2, "context")),
    ];

    let split_rows = DiffSplitModelOps::build(&rows);

    assert_eq!(split_rows.len(), 2);
    assert!(
        matches!(&split_rows[0], SplitDiffRow::Line(line) if line.before.is_some() && line.after.is_some())
    );
    assert_eq!(
        split_rows[1],
        SplitDiffRow::Line(SplitDiffLine {
            before: Some(DiffCell {
                line_number: 2,
                text: "context".to_string(),
                kind: DiffLineKind::Unchanged,
                highlight_ranges: vec![],
            }),
            after: Some(DiffCell {
                line_number: 2,
                text: "context".to_string(),
                kind: DiffLineKind::Unchanged,
                highlight_ranges: vec![],
            }),
        })
    );
}

#[test]
fn split_model_defers_weak_anchor_between_changed_lines() {
    let rows = vec![
        InlineDiffRow::Line(DiffLine::removed(1, "old")),
        InlineDiffRow::Line(DiffLine::unchanged(2, 2, "")),
        InlineDiffRow::Line(DiffLine::added(1, "new")),
        InlineDiffRow::Line(DiffLine::unchanged(3, 3, "tail")),
    ];

    let split_rows = DiffSplitModelOps::build(&rows);

    assert_eq!(split_rows.len(), 4);
    assert!(
        matches!(&split_rows[0], SplitDiffRow::Line(line) if line.before.is_some() && line.after.is_none())
    );
    assert!(
        matches!(&split_rows[1], SplitDiffRow::Line(line) if line.before.is_none() && line.after.is_some())
    );
    assert!(
        matches!(&split_rows[2], SplitDiffRow::Line(line) if line.before.as_ref().is_some_and(|it| it.text.is_empty()))
    );
    assert!(
        matches!(&split_rows[3], SplitDiffRow::Line(line) if line.before.as_ref().is_some_and(|it| it.text == "tail"))
    );
}

#[test]
fn split_pairing_does_not_pair_empty_similarity_without_fallback() {
    let removed = vec![DiffLine::removed(1, "")];
    let added = vec![DiffLine::added(1, "new")];

    assert_eq!(
        DiffSplitPairingOps::line_pairs(&removed, &added, false),
        vec![None]
    );
}
