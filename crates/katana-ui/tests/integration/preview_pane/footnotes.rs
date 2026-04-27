use eframe::egui;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const PANEL_WIDTH: f32 = 1000.0;
const PANEL_HEIGHT: f32 = 8000.0;

fn load_sample_sections() -> Vec<RenderedSection> {
    /* WHY: Utility to load and render the standard sample.md fixture for footnote verification. */
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.md");
    let mut pane = PreviewPane::default();
    crate::integration::test_helpers::MissingRendererAssetsOps::with(|| {
        pane.full_render(
            &source,
            &path,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            false,
            2,
        );
        pane.wait_for_renders();
    });
    std::mem::take(&mut pane.sections)
}

fn build_harness(sections: Vec<RenderedSection>) -> Harness<'static> {
    /* WHY: Factory to create a UI harness with pre-rendered sections for footnote testing. */
    let mut harness = Harness::builder()
        .with_size(egui::vec2(PANEL_WIDTH, PANEL_HEIGHT))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui, None, None, None, None);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness
}

#[test]
fn bug2_footnote_definitions_must_be_at_document_end() {
    /* WHY: Regression test (Bug 2): Ensure that footnote definitions are globally aggregated
     * and rendered at the very end of the document, even when sections are rendered independently. */
    let harness = build_harness(load_sample_sections());

    /* WHY: "there are no rendering regressions." is the very last text line in sample.md
    (inside "## ✅ Verification Complete"). Footnote definitions MUST come after this. */
    let footnote_def = harness.get_by_label_contains("First footnote content.");
    let last_doc_text = harness.get_by_label_contains("there are no rendering regressions");

    let footnote_bounds = footnote_def
        .accesskit_node()
        .raw_bounds()
        .expect("footnote definition must have layout bounds");
    let last_bounds = last_doc_text
        .accesskit_node()
        .raw_bounds()
        .expect("last document text must have layout bounds");

    assert!(
        footnote_bounds.y0 > last_bounds.y0,
        "Bug 2 (footnote not at end): footnote definition (y0={:.1}) appears before \
         the last document text (y0={:.1}).",
        footnote_bounds.y0,
        last_bounds.y0,
    );
}

#[test]
fn bug5_footnote_references_must_not_render_as_raw_text() {
    /* WHY: Regression test (Bug 5): Ensure footnote references like '[^1]' are correctly parsed
     * and rendered as interactive links instead of visible raw Markdown text. */
    let harness = build_harness(load_sample_sections());

    let raw_nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("[^1]"))
        .collect();

    assert!(
        raw_nodes.is_empty(),
        "Regression (Bug 5): Footnote reference '[^1]' is exposed as literal raw text \
         ({} nodes found). \
         Footnote references must be rendered as clickable footnote links, not raw text.",
        raw_nodes.len()
    );
}

#[test]
fn bug6_footnote_bidirectional_links_exist() {
    /* WHY: Regression test (Bug 6): Verify that footnote definitions contain the return symbol ('↩'),
     * which is essential for bidirectional navigation between text and citations. */
    let harness = build_harness(load_sample_sections());

    let return_links: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("↩"))
        .collect();

    assert!(
        !return_links.is_empty(),
        "Regression (Bug 6): No return links ('↩') found for footnotes. \
        Bidirectional linking from footnote definition strictly requires these to exist."
    );
}
