use katana_core::markdown::{ExportFormat, ExportInput, ExporterTrait, ImageExporter, PdfExporter};

fn do_image_export(html: &str, output: &std::path::Path) {
    ImageExporter
        .export(&ExportInput {
            format: ExportFormat::Png,
            html_source: html.to_string(),
            output_path: output.to_path_buf(),
            config: Default::default(),
        })
        .unwrap();
}

fn do_pdf_export(html: &str, output: &std::path::Path) {
    PdfExporter
        .export(&ExportInput {
            format: ExportFormat::Pdf,
            html_source: html.to_string(),
            output_path: output.to_path_buf(),
            config: Default::default(),
        })
        .unwrap();
}

#[test]
fn image_export_preserves_html_body_background_in_png() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(dark_export_html(), &output);

    let image = image::open(output).unwrap().to_rgba8();
    assert_dark_pixel(image.get_pixel(8, 8).0);
}

#[test]
fn pdf_export_preserves_html_body_background_in_embedded_image() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.pdf");

    do_pdf_export(dark_export_html(), &output);

    let jpeg = first_pdf_stream(&std::fs::read(output).unwrap());
    let image = image::load_from_memory(&jpeg).unwrap().to_rgba8();
    assert_dark_pixel(image.get_pixel(8, 8).0);
}

#[test]
fn image_export_normalizes_percent_width_svg_before_embedding() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(percent_width_svg_html(), &output);

    let image = image::open(output).unwrap().to_rgba8();
    let far_right_pixel = image.get_pixel(850, 90).0;
    assert_dark_pixel(far_right_pixel);
}

fn first_pdf_stream(pdf: &[u8]) -> Vec<u8> {
    let start_marker = b"stream\n";
    let end_marker = b"\nendstream";
    let start = pdf
        .windows(start_marker.len())
        .position(|it| it == start_marker)
        .unwrap()
        + start_marker.len();
    let end = pdf[start..]
        .windows(end_marker.len())
        .position(|it| it == end_marker)
        .unwrap()
        + start;
    pdf[start..end].to_vec()
}

fn assert_dark_pixel(pixel: [u8; 4]) {
    assert!(
        pixel[0] < 60 && pixel[1] < 60 && pixel[2] < 60,
        "native export must keep the HTML body background; got rgba({},{},{},{})",
        pixel[0],
        pixel[1],
        pixel[2],
        pixel[3]
    );
}

fn dark_export_html() -> &'static str {
    r##"<!DOCTYPE html>
<html>
<head>
<style>
body { font-family: Arial, sans-serif; background-color: #1e1e1e; color: #E0E0E0; }
</style>
</head>
<body>
<h1>Dark Export</h1>
<div class="katana-diagram mermaid">
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
<rect x="8" y="8" width="104" height="64" fill="#2D2D2D" stroke="#AAAAAA"/>
<text x="24" y="46" fill="#E0E0E0">Diagram</text>
</svg>
</div>
</body>
</html>"##
}

fn percent_width_svg_html() -> &'static str {
    r##"<!DOCTYPE html>
<html>
<head>
<style>
body { background-color: #1e1e1e; color: #E0E0E0; }
</style>
</head>
<body>
<svg xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 120 80">
<rect x="0" y="0" width="120" height="80" fill="#ff0000"/>
</svg>
</body>
</html>"##
}
