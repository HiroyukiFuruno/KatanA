use std::path::Path;

#[test]
fn bug1_html_export_must_transform_diagram_blocks_into_html_images_or_errors() {
    /* WHY: Regression test (Bug 1): Verify that diagram fences (mermaid/drawio/plantuml) are correctly
     * transformed into HTML image or error divs during export, instead of being left as raw Markdown code blocks. */
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.ja.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.ja.md");

    let preset = katana_core::markdown::color_preset::DiagramColorPreset::default();
    let renderer = katana_core::markdown::KatanaRenderer;
    let exported_html =
        katana_core::markdown::HtmlExporter::export(&source, &renderer, &preset, None)
            .expect("Html exporter should succeed");

    /* WHY: If transformation fails, comrak converts the remainig fence to `<code class="language-mermaid">`.
     * We must ensure no such raw blocks exist in the final output. */
    let has_raw_mermaid_block = exported_html.contains("language-mermaid");
    let has_raw_drawio_block = exported_html.contains("language-drawio");
    let has_raw_plantuml_block = exported_html.contains("language-plantuml");

    assert!(
        !has_raw_mermaid_block && !has_raw_drawio_block && !has_raw_plantuml_block,
        "Bug 1 (HTML export diagrams failure): Exported HTML contains raw diagram code blocks."
    );
}
