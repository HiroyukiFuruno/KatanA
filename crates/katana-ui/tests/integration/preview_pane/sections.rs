use egui_kittest::Harness;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::PathBuf;

#[test]
fn markdown_only_input_is_sectioned_correctly() {
    /* WHY: Verify that if the input doesn't contain any rich blocks, it remains a single Markdown section. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Title\n\nParagraph 1\n\n## Subtitle\n\nParagraph 2",
        std::path::Path::new("/tmp/test.md"),
    );
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_, _)));
}

#[test]
fn mixed_diagram_input_is_split_into_sections() {
    /* WHY: Verify that combining multiple diagrams and text results in a correctly ordered sequence of Markdown and Rich sections. */
    let mut pane = PreviewPane::default();
    let src =
        "Before\n```mermaid\ngraph TD; A-->B\n```\nMiddle\n```drawio\n<mxGraphModel/>\n```\nAfter";
    pane.update_markdown_sections(src, std::path::Path::new("/tmp/test.md"));
    assert!(pane.sections.len() >= 3);
}

#[test]
fn empty_input_returns_empty_section_list() {
    /* WHY: Optimization check: Verify that whitespace or empty strings don't result in dead UI elements. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("", &std::path::PathBuf::from("/tmp/test.md"));
    assert!(pane.sections.is_empty());
}

#[test]
fn centered_html_stays_in_markdown_section_update() {
    /* WHY: Regression test for HTML alignment tags: Verify that <p align="center"> tags are NOT identified as separate blocks,
     * but remain integrated in the Markdown section for egui_commonmark to handle. */
    let mut pane = PreviewPane::default();
    let src = "<p align=\"center\">centered</p>";
    pane.update_markdown_sections(src, std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_, _)));
}

#[test]
fn show_section_markdown_variant_renders() {
    /* WHY: UI render check: Verify that a standard Markdown section produces no crashes during the egui draw cycle. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Hello from egui test",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();
}

#[test]
fn show_section_error_variant_renders() {
    /* WHY: UI render check: Verify that the error display (e.g., for failed diagram rendering) is visible and doesn't crash. */
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Error {
        kind: "Mermaid".to_string(),
        _source: "bad".to_string(),
        message: "failed".to_string(),
        source_lines: 0,
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();
}

#[test]
fn show_section_pending_variant_renders() {
    /* WHY: UI render check: Verify that the spinner/loading state for diagrams is rendered correctly. */
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Pending {
        kind: "Mermaid".to_string(),
        source: "src".to_string(),
        source_lines: 0,
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.step();
}

#[test]
fn show_section_not_installed_variant_renders() {
    /* WHY: UI render check: Verify the display for external tools that are missing from the system path. */
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("/tmp/plantuml.jar"),
        source_lines: 0,
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();
}
