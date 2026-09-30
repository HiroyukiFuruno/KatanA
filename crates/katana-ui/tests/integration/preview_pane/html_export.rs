use std::path::Path;

use crate::integration::harness_utils::flatten_shapes;
use eframe::egui;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};

#[test]
fn bug1_html_export_must_transform_diagram_blocks_into_html_images_or_errors() {
    /* WHY: Regression test (Bug 1): Verify that diagram fences (mermaid/drawio/plantuml) are correctly
     * transformed into HTML image or error divs during export, instead of being left as raw Markdown code blocks. */
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.ja.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.ja.md");

    let preset = katana_core::markdown::color_preset::DiagramColorPreset::default();
    let exported_html = crate::integration::test_helpers::MissingRendererAssetsOps::with(|| {
        let exporter = katana_core::markdown::HtmlExporter;
        exporter
            .export_markdown_to_html(&source, &preset, None)
            .expect("Html exporter should succeed")
    });

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

#[test]
fn quoted_data_uri_html_image_renders_without_svg_text_leak() {
    let src = r#"data:image/svg+xml,%3Csvg xmlns=%22<http://www.w3.org/2000/svg%22> width=%22128%22%3E%3C/svg%3E"#;
    let markdown =
        format!(r#"<p align="center"><img src="{src}" width="128" alt="KatanA icon"></p>"#);
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(&markdown, Path::new("/tmp/sample.md"));
    let [RenderedSection::Markdown(section, _)] = pane.sections.as_slice() else {
        panic!("quoted data URI must remain a single host preview section");
    };
    assert_eq!(section, &markdown);

    let ctx = egui::Context::default();
    let output = crate::IntegrationUiOps::run(&ctx, egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            pane.show_content(ui, None, None, None, None);
        });
    });

    let shapes: Vec<egui::Shape> = output.shapes.into_iter().map(|shape| shape.shape).collect();
    let shapes = flatten_shapes(&shapes);
    assert!(
        !shapes.iter().any(|shape| {
            matches!(shape, egui::Shape::Text(text) if {
                let rendered = &text.galley.job.text;
                rendered.contains("%3Csvg") || rendered.contains("width=%22128%22%3E")
            })
        }),
        "quoted data URI SVG markup must not leak as preview text"
    );
}
