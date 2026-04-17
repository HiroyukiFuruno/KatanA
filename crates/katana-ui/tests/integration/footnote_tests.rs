use eframe::egui;
use egui_kittest::{Harness, kittest::{NodeT, Queryable}};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const PANEL_WIDTH: f32 = 1000.0;
const PANEL_HEIGHT: f32 = 8000.0;

fn load_sample_sections() -> Vec<RenderedSection> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.ja.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.ja.md");
    let mut pane = PreviewPane::default();
    pane.full_render(
        &source,
        &path,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        2,
    );
    pane.wait_for_renders();
    std::mem::take(&mut pane.sections)
}

fn build_harness(sections: Vec<RenderedSection>) -> Harness<'static> {
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
fn bug5_footnote_references_must_not_render_as_raw_text() {
    let harness = build_harness(load_sample_sections());

    // In sample.ja.md, the text contains a footnote reference: "[^1]".
    // If the footnote reference is broken, the literal "[^1]" will be exposed.
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
    let harness = build_harness(load_sample_sections());

    // We should be able to find the footnote link in the main text.
    // In egui_commonmark, a footnote link typically is rendered as a hoverable/clickable element.
    // The exact text rendered for footnote 1 is "1".
    // 
    // And there should be a return link "↩" rendered in the footnote definition.
    let return_links: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("↩"))
        .collect();

    assert!(
        !return_links.is_empty(),
        "Regression (Bug 6): No return links ('↩') found for footnotes. \
        Bidirectional linking from footnote definition strictly requires these to exist."
    );
}
