use super::{DiffModelOps, SplitDiffRow};

#[test]
fn build_pairs_similar_lines_across_blank_anchor() {
    let before = concat!(
        "This returns:\n",
        "--- Context file paths vary by schema\n",
        "--- Progress total complete remaining\n",
        "\n",
        "Done",
    );
    let after = concat!(
        "This returns:\n",
        "\n",
        "--- Context file paths vary by schema and docs\n",
        "--- Progress total complete remaining now\n",
        "Done",
    );
    let model = DiffModelOps::build(before, after);

    assert_paired_texts(
        &model.split_rows[1],
        "--- Context file paths vary by schema",
        "--- Context file paths vary by schema and docs",
    );
    assert_paired_texts(
        &model.split_rows[2],
        "--- Progress total complete remaining",
        "--- Progress total complete remaining now",
    );
}

fn assert_paired_texts(row: &SplitDiffRow, before: &str, after: &str) {
    let SplitDiffRow::Line(line) = row else {
        panic!("expected split line");
    };

    assert_eq!(
        line.before.as_ref().map(|cell| cell.text.as_str()),
        Some(before),
        "{row:#?}"
    );
    assert_eq!(
        line.after.as_ref().map(|cell| cell.text.as_str()),
        Some(after),
        "{row:#?}"
    );
}
