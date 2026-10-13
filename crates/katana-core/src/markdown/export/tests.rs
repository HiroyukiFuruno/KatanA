use super::*;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramThemeOverride, DiagramThemeSnapshot};

const DRAWIO_SOURCE: &str = r#"<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Hello" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>"#;

fn diagram_markdown() -> String {
    format!(
        "# Export\n\n```mermaid\ngraph TD\n    A[Start] --> B[End]\n```\n\n```drawio\n{DRAWIO_SOURCE}\n```\n"
    )
}

const LONG_PROFILE_URL: &str = "https://www.linkedin.com/in/%E8%A3%95%E4%B9%8B-%E5%8F%A4%E9%87%8E-a00785199/very-long-link-target-segment-for-pdf-wrap-verification-very-long-link-target-segment-for-pdf-wrap-verification";

fn markdown_fidelity_source() -> String {
    format!(
        "# Export fidelity\n\n\
         A normal markdown link: [KatanA](https://example.com/katana)\n\n\
         Long visible link: [{LONG_PROFILE_URL}]({LONG_PROFILE_URL})\n\n\
         | Feature | Expected | Link |\n\
         | --- | --- | --- |\n\
         | PDF table | rendered as table grid with header/body styling | [PDF docs](https://example.com/pdf) |\n"
    )
}

fn diagram_html() -> String {
    let preset = DiagramColorPreset::default();
    HtmlExporter
        .export_markdown_to_html(&diagram_markdown(), &preset, None)
        .expect("diagram HTML export should succeed")
}

fn export_bytes(format: ExportFormat, extension: &str, source: &str) -> Vec<u8> {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let output_path = temp_dir.path().join(format!("diagram.{extension}"));
    let input = ExportInput {
        format,
        markdown_source: source.to_string(),
        source_path: temp_dir.path().join("diagram.md"),
        output_path: output_path.clone(),
        config: ExportConfig::default(),
    };

    match format {
        ExportFormat::Pdf => PdfExporter.export(&input),
        ExportFormat::Png | ExportFormat::Jpeg => ImageExporter.export(&input),
        ExportFormat::Html => HtmlExporter.export(&input),
    }
    .expect("native export should succeed");

    std::fs::read(output_path).expect("exported file should be readable")
}

#[test]
fn export_config_default_uses_current_theme_override() {
    DiagramThemeSnapshot::set_current_override(DiagramThemeOverride {
        name: "pdf-export-test".to_string(),
        is_dark: false,
        background: "#FAFAFA".to_string(),
        text: "#121212".to_string(),
        preview_text: "#343434".to_string(),
        table_border: Some("#808080".to_string()),
        table_header_background: Some("#d5e8ff".to_string()),
        table_even_row_background: Some("#f2f7ff".to_string()),
    });

    let config = ExportConfig::default();
    DiagramThemeSnapshot::clear_current_override();

    assert_eq!(config.theme.background, "#FAFAFA");
    assert_eq!(config.theme.text, "#121212");
    assert_eq!(config.theme.table_border.as_deref(), Some("#808080"));
    assert_eq!(
        config.theme.table_header_background.as_deref(),
        Some("#d5e8ff")
    );
    assert_eq!(
        config.theme.table_even_row_background.as_deref(),
        Some("#f2f7ff")
    );
}

#[test]
fn html_export_transforms_mermaid_and_drawio_blocks() {
    let html = diagram_html();

    assert!(html.contains("<svg"));
    assert!(!html.contains("language-mermaid"));
    assert!(!html.contains("language-drawio"));
}

#[test]
fn html_export_preserves_markdown_links_and_tables() {
    let source = markdown_fidelity_source();
    let html = String::from_utf8(export_bytes(ExportFormat::Html, "html", &source))
        .expect("HTML export should be UTF-8");

    assert!(
        html.contains("<a href=\"https://example.com/katana\""),
        "HTML export must preserve normal markdown links as anchors: {html}"
    );
    assert!(
        html.contains("<table"),
        "HTML export must preserve markdown tables as table markup: {html}"
    );
    assert!(
        html.contains("<a href=\"https://example.com/pdf\""),
        "HTML export must preserve links inside table cells: {html}"
    );
    assert!(
        html.contains(&format!("<a href=\"{LONG_PROFILE_URL}\"")),
        "HTML export must preserve long visible links as anchors: {html}"
    );
}

#[test]
fn pdf_export_preserves_markdown_link_annotations() {
    let source = markdown_fidelity_source();
    let pdf = export_bytes(ExportFormat::Pdf, "pdf", &source);
    let text = String::from_utf8_lossy(&pdf);

    assert!(pdf.starts_with(b"%PDF"));
    assert!(
        text.contains("/Subtype /Link"),
        "PDF export must preserve markdown links as PDF link annotations"
    );
    assert!(
        text.matches(LONG_PROFILE_URL).count() >= 2,
        "PDF export must preserve long wrapped link annotations for each visible segment"
    );
}

#[test]
fn native_exporters_write_pdf_png_and_jpeg_from_diagram_html() {
    let source = diagram_markdown();

    let pdf = export_bytes(ExportFormat::Pdf, "pdf", &source);
    let png = export_bytes(ExportFormat::Png, "png", &source);
    let jpeg = export_bytes(ExportFormat::Jpeg, "jpg", &source);

    assert!(pdf.starts_with(b"%PDF"));
    assert!(png.starts_with(b"\x89PNG"));
    assert!(jpeg.starts_with(&[0xFF, 0xD8]));
}
