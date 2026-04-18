use katana_ui::preview_pane::{PreviewPane, RenderedSection};

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
fn image_path_resolved_in_rendered_markdown_section() {
    /* WHY: Verify that relative image paths in Markdown are resolved to absolute file:// URIs using the document's path as a base. */
    let dir = tempfile::tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    let assets_dir = dir.path().join("assets");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::create_dir_all(&assets_dir).unwrap();
    std::fs::write(assets_dir.join("logo.png"), b"fake-png").unwrap();

    let md_path = docs_dir.join("readme.md");
    let source = "# Title\n\nInline image: ![logo](../assets/logo.png)\n\nAfter image.";

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, &md_path);

    let texts = markdown_texts(&pane.sections);
    assert_eq!(texts.len(), 1);
    assert!(
        texts[0].contains("file://"),
        "Image path should be resolved to file:// URI, got: {}",
        texts[0]
    );
    assert!(
        !texts[0].contains("../"),
        "Relative path '..' should be resolved, got: {}",
        texts[0]
    );
}

#[test]
fn http_image_url_preserved_in_rendered_markdown_section() {
    /* WHY: Verify that external HTTP/HTTPS image URLs are left unchanged by the path resolution logic. */
    let source = "# Title\n\nInline: ![logo](https://example.com/logo.png)\n";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, std::path::Path::new("/tmp/test.md"));

    let texts = markdown_texts(&pane.sections);
    assert!(
        texts[0].contains("https://example.com/logo.png"),
        "HTTP URL should be preserved unchanged, got: {}",
        texts[0]
    );
}

#[test]
fn standalone_local_image_is_split_into_local_image_section() {
    /* WHY: Verify that standalone images (on their own line) are identified as LocalImage sections, 
     * enabling the UI to render them with specific layout features (e.g., zoom/centered). */
    let source = "# Title\n\n![Logo](file:///path/to/logo.png)\n\nText";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, std::path::Path::new("/tmp/test.md"));

    assert_eq!(pane.sections.len(), 3);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_, _)));
    if let RenderedSection::LocalImage { path, alt, .. } = &pane.sections[1] {
        assert_eq!(path.to_string_lossy(), "/path/to/logo.png");
        assert_eq!(alt, "Logo");
    } else {
        panic!("Expected LocalImage section at index 1");
    }
}
