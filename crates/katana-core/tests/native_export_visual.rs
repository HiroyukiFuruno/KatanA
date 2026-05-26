use katana_core::markdown::{
    ExportConfig, ExportFormat, ExportInput, ExporterTrait, ImageExporter, PdfExporter,
    color_preset::DiagramColorPreset,
};

fn do_image_export(markdown: &str, output: &std::path::Path) {
    ImageExporter
        .export(&ExportInput {
            format: ExportFormat::Png,
            markdown_source: markdown.to_string(),
            source_path: output.with_extension("md"),
            output_path: output.to_path_buf(),
            config: dark_export_config(),
        })
        .unwrap();
}

fn do_pdf_export(markdown: &str, output: &std::path::Path) {
    PdfExporter
        .export(&ExportInput {
            format: ExportFormat::Pdf,
            markdown_source: markdown.to_string(),
            source_path: output.with_extension("md"),
            output_path: output.to_path_buf(),
            config: dark_export_config(),
        })
        .unwrap();
}

#[test]
fn image_export_writes_markdown_content_pixels_in_png() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(dark_export_markdown(), &output);

    let image = image::open(output).unwrap().to_rgba8();
    assert_has_nonwhite_pixel(&image);
}

#[test]
fn pdf_export_writes_pdf_page_from_markdown() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.pdf");

    do_pdf_export(dark_export_markdown(), &output);

    let bytes = std::fs::read(output).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(
        bytes
            .windows(b"/Type /Page".len())
            .any(|it| it == b"/Type /Page")
    );
}

#[test]
fn image_export_handles_percent_width_svg_markdown_without_chromium() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(percent_width_svg_markdown(), &output);

    let image = image::open(output).unwrap().to_rgba8();
    assert!(image.width() > 0);
    assert!(image.height() > 0);
}

fn assert_has_nonwhite_pixel(image: &image::RgbaImage) {
    assert!(
        image
            .pixels()
            .any(|pixel| pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250),
        "native export must render visible markdown content"
    );
}

fn dark_export_config() -> ExportConfig {
    ExportConfig {
        theme: DiagramColorPreset::dark().clone(),
        ..Default::default()
    }
}

fn dark_export_markdown() -> &'static str {
    r##"# Dark Export

<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
<rect x="8" y="8" width="104" height="64" fill="#2D2D2D" stroke="#AAAAAA"/>
<text x="24" y="46" fill="#E0E0E0">Diagram</text>
</svg>
"##
}

fn percent_width_svg_markdown() -> &'static str {
    r##"# Percent Width SVG

<svg xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 120 80">
<rect x="0" y="0" width="120" height="80" fill="#ff0000"/>
</svg>
"##
}
