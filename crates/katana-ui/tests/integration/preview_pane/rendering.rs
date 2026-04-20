use katana_platform::InMemoryCacheService;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::sync::Arc;

fn markdown_texts(sections: &[RenderedSection]) -> Vec<&str> {
    sections
        .iter()
        .filter_map(|s| match s {
            RenderedSection::Markdown(md, _) => Some(md.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn unsaved_buffer_changes_are_reflected_in_preview() {
    /* WHY: Verify that current buffer content (not necessarily saved to disk) is rendered in the preview. */
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections("# Hello", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello"));

    pane.update_markdown_sections(
        "# Hello World\n\nNew paragraph",
        std::path::Path::new("/tmp/test.md"),
    );
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello World"));
    assert!(texts[0].contains("New paragraph"));
}

#[test]
fn consecutive_edits_are_immediately_reflected_in_preview() {
    /* WHY: Verify that high-frequency updates (simulating typing) correctly update the preview state without dropouts or stale content. */
    let mut pane = PreviewPane::default();

    let edits = vec![
        "# Draft 1",
        "# Draft 2\n\n- item A",
        "# Draft 3\n\n- item A\n- item B\n- item C",
    ];

    for edit in &edits {
        pane.update_markdown_sections(edit, std::path::Path::new("/tmp/test.md"));
        let texts = markdown_texts(&pane.sections);
        assert!(
            texts[0].contains(edit),
            "Edit not reflected in preview: {edit}"
        );
    }
}

#[test]
fn empty_buffer_does_not_crash_preview() {
    /* WHY: Verify that clearing the buffer results in an empty section list rather than errors or stale content. */
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections("# Hello", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);

    pane.update_markdown_sections("", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 0);
}

#[test]
fn buffer_with_diagrams_immediately_updates_markdown_portion_only() {
    /* WHY: Verify that full_render correctly handles multi-step rendering:
     * Markdown text portion remains responsive while complex diagrams (Mermaid) are in 'Pending' state. */
    let mut pane = PreviewPane::default();

    let source = "# Title\n```mermaid\ngraph TD; A-->B\n```\n## Footer";
    pane.full_render(
        source,
        std::path::Path::new("/tmp/test.md"),
        Arc::new(InMemoryCacheService::default()),
        false,
        4,
    );

    assert!(pane.sections.len() >= 3);
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));

    let modified = "# Updated Title\n```mermaid\ngraph TD; A-->B\n```\n## Updated Footer";
    pane.update_markdown_sections(modified, std::path::Path::new("/tmp/test.md"));

    let texts = markdown_texts(&pane.sections);
    assert!(texts.iter().any(|t| t.contains("Updated Title")));
    assert!(texts.iter().any(|t| t.contains("Updated Footer")));
}

#[test]
fn full_render_splits_sections_correctly() {
    /* WHY: Verify that diagrams (Mermaid blocks) are correctly identified and isolated into their own sections,
     * separating preceding and following markdown. */
    let mut pane = PreviewPane::default();

    let source = "Before\n```mermaid\ngraph TD; A-->B\n```\nAfter";
    pane.full_render(
        source,
        std::path::Path::new("/tmp/test.md"),
        Arc::new(InMemoryCacheService::default()),
        false,
        4,
    );

    assert_eq!(pane.sections.len(), 3);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_, _)));
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));
    assert!(matches!(pane.sections[2], RenderedSection::Markdown(_, _)));
}

#[test]
fn buffer_without_diagrams_does_not_generate_pending_sections() {
    /* WHY: Optimization check: Verify that if no complex diagrams are present, all sections are plain Markdown immediately. */
    let mut pane = PreviewPane::default();

    pane.full_render(
        "# Pure Markdown\n\nNo diagrams here.",
        std::path::Path::new("/tmp/test.md"),
        Arc::new(InMemoryCacheService::default()),
        false,
        4,
    );

    assert!(
        pane.sections
            .iter()
            .all(|s| matches!(s, RenderedSection::Markdown(_, _)))
    );
    assert!(
        !pane
            .sections
            .iter()
            .any(|s| matches!(s, RenderedSection::Pending { .. }))
    );
}

#[test]
fn verification_that_preview_updates_do_not_depend_on_file_saves() {
    /* WHY: Verify the core architectural requirement that the UI state (PreviewPane) is tied to
     * the in-memory Document buffer rather than the file on disk. */
    use katana_core::document::Document;

    let mut doc = Document::new("/workspace/spec.md", "# Original");
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections(&doc.buffer, std::path::Path::new("/tmp/test.md"));
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Original"));

    doc.update_buffer("# Modified by user\n\nThis is not saved yet.");
    assert!(doc.is_dirty, "Document must be dirty");

    pane.update_markdown_sections(&doc.buffer, std::path::Path::new("/tmp/test.md"));
    let texts = markdown_texts(&pane.sections);
    assert!(
        texts[0].contains("Modified by user"),
        "Unsaved edits are not reflected in preview"
    );

    assert!(doc.is_dirty, "Document should not have been saved");
}
