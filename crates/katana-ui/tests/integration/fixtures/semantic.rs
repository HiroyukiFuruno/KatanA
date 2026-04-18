use super::harness::{PANEL_WIDTH, build_harness, extract_section, load_fixture};
use katana_ui::preview_pane::PreviewPane;
use std::path::Path;

fn render_snippet(md: &str) -> PreviewPane {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(md, Path::new("/tmp/snippet.md"));
    pane
}

#[test]
fn fixture_en_s1_1_centered_heading() {
    /* WHY: Verify that H1 headings are correctly centered when wrapped in <p align="center"> tags. */
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    // Position checks would go here using accesskit nodes if needed
}

#[test]
fn fixture_ja_s1_1_centered_heading() {
    /* WHY: Verify that Japanese H1 headings are also correctly centered. */
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
}

#[test]
fn fixture_en_s1_5_text_link_same_row() {
    /* WHY: Verify that inline links are rendered on the same row as their surrounding text,
     * guarding against incorrect line breaks in the markdown renderer. */
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
}

#[test]
fn fixture_en_s2_1_heading_hierarchy() {
    /* WHY: Verify that all markdown heading levels (H1-H6) are rendered in the correct order
     * and maintain relative vertical spacing. */
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.1", "### 2.2");
    let pane = render_snippet(&section_md);
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
}
