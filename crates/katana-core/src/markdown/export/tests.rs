use super::*;
use crate::markdown::color_preset::DiagramColorPreset;

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

fn diagram_html() -> String {
    let preset = DiagramColorPreset::default();
    HtmlExporter
        .export_markdown_to_html(&diagram_markdown(), &preset, None)
        .expect("diagram HTML export should succeed")
}

fn export_bytes(format: ExportFormat, extension: &str, html: &str) -> Vec<u8> {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let output_path = temp_dir.path().join(format!("diagram.{extension}"));
    let input = ExportInput {
        format: format.clone(),
        html_source: html.to_string(),
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
fn html_export_transforms_mermaid_and_drawio_blocks() {
    let html = diagram_html();

    assert!(html.contains("<svg"));
    assert!(!html.contains("language-mermaid"));
    assert!(!html.contains("language-drawio"));
}

#[test]
fn native_exporters_write_pdf_png_and_jpeg_from_diagram_html() {
    let html = diagram_html();

    let pdf = export_bytes(ExportFormat::Pdf, "pdf", &html);
    let png = export_bytes(ExportFormat::Png, "png", &html);
    let jpeg = export_bytes(ExportFormat::Jpeg, "jpg", &html);

    assert!(pdf.starts_with(b"%PDF"));
    assert!(png.starts_with(b"\x89PNG"));
    assert!(jpeg.starts_with(&[0xFF, 0xD8]));
}
