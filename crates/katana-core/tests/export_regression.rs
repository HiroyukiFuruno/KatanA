use katana_core::markdown::{
    DiagramBlock, DiagramRenderer, DiagramResult, ExportFormat, ExportInput, ExporterTrait,
    ImageExporter, MarkdownError, MarkdownRenderOps, PdfExporter, RenderOutput,
};
use std::collections::HashSet;
use std::sync::Mutex;

fn do_pdf_export(html: &str, output: &std::path::Path) {
    PdfExporter
        .export(&ExportInput {
            format: ExportFormat::Pdf,
            html_source: html.to_string(),
            output_path: output.to_path_buf(),
        })
        .unwrap();
}

fn do_image_export(html: &str, output: &std::path::Path) {
    ImageExporter
        .export(&ExportInput {
            format: ExportFormat::Png,
            html_source: html.to_string(),
            output_path: output.to_path_buf(),
        })
        .unwrap();
}

static RENDER_ENV_LOCK: Mutex<()> = Mutex::new(());

struct DummyRenderer;

impl DiagramRenderer for DummyRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        DiagramResult::Ok(format!(
            "<div class=\"katana-diagram\">Dummy Diagram {}</div>",
            block.source.trim()
        ))
    }
}

fn render_with_missing_diagram_assets(source: &str) -> Result<RenderOutput, MarkdownError> {
    let _guard = RENDER_ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("MERMAID_JS", dir.path().join("missing-mermaid.min.js")) };
    unsafe { std::env::set_var("DRAWIO_JS", dir.path().join("missing-drawio.min.js")) };
    unsafe { std::env::set_var("PLANTUML_JAR", dir.path().join("missing-plantuml.jar")) };
    let output = MarkdownRenderOps::render_with_katana_renderer(source);
    unsafe { std::env::remove_var("MERMAID_JS") };
    unsafe { std::env::remove_var("DRAWIO_JS") };
    unsafe { std::env::remove_var("PLANTUML_JAR") };
    output
}

#[test]
fn test_html_export_diagram_heading_swallow() {
    let source = "
```mermaid
graph TD; A-->B
```
## Next Heading
";
    let output = MarkdownRenderOps::render(source, &DummyRenderer).unwrap();
    // The rendered HTML must contain an h2 tag for the Next Heading!
    assert!(
        output.html.contains("<h2"),
        "HTML output did not contain the <h2> for Next Heading! This proves the heading was swallowed by the preceding diagram's HTML block!"
    );
}

/// RED test: Verify the REAL KatanaRenderer produces diagram output (not raw code blocks).
/// This test uses the actual rendering pipeline that users experience.
#[test]
fn html_export_with_katana_renderer_must_not_produce_raw_code() {
    let source = r#"
```mermaid
graph TD; A-->B
```

## After Diagram
"#;
    let output = render_with_missing_diagram_assets(source).unwrap();

    // If KatanaRenderer processed the block, it should produce EITHER:
    // - A rendered image (contains "katana-diagram" or "data:image/png")
    // - A fallback error (contains "katana-diagram-error")
    // It must NEVER produce a raw <code> block with the source.
    let has_diagram_output =
        output.html.contains("katana-diagram") || output.html.contains("data:image/png");
    let has_error_output = output.html.contains("katana-diagram-error");
    let has_raw_code = output.html.contains("language-mermaid");

    eprintln!("=== ACTUAL HTML OUTPUT ===");
    eprintln!("{}", output.html);
    eprintln!("=== END ===");

    assert!(
        has_diagram_output || has_error_output,
        "KatanaRenderer did not produce diagram output or error fallback. \
         The diagram was likely skipped by transform_diagram_blocks."
    );

    assert!(
        !has_raw_code,
        "KatanaRenderer produced raw code blocks — diagrams are not being processed!"
    );
}

/// RED test: Consecutive diagrams must ALL be processed (not just the first one).
#[test]
fn html_export_consecutive_diagrams_all_processed() {
    let source = r#"
## Consecutive Diagrams

```mermaid
pie title Test
    "A" : 1
```

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
  </root>
</mxGraphModel>
```

```plantuml
@startuml
Alice -> Bob : OK
@enduml
```

End.
"#;
    let output = render_with_missing_diagram_assets(source).unwrap();

    eprintln!("=== CONSECUTIVE DIAGRAMS OUTPUT ===");
    eprintln!("{}", output.html);
    eprintln!("=== END ===");

    // None of the three diagram sources should appear as raw code
    assert!(
        !output.html.contains("language-mermaid"),
        "Mermaid block was not processed — showed as raw code"
    );
    assert!(
        !output.html.contains("language-drawio"),
        "DrawIO block was not processed — showed as raw code"
    );
    assert!(
        !output.html.contains("language-plantuml"),
        "PlantUML block was not processed — showed as raw code"
    );

    // The heading and trailing text must survive
    assert!(output.html.contains("<h2"), "Heading was swallowed");
    assert!(output.html.contains("End."), "Trailing text was swallowed");
}

/// RED test for Bug 3: Inline math with spaces must be rendered
#[test]
fn html_export_must_render_inline_math_with_spaces_correctly() {
    let source = "Inline math: $ E = mc^2 $ should work.";
    let output = MarkdownRenderOps::render_with_katana_renderer(source).unwrap();

    eprintln!("=== INLINE MATH OUTPUT ===");
    eprintln!("{}", output.html);
    eprintln!("=== END ===");

    // comrak outputs Math as something like: <span data-math-style="inline">E = mc^2</span>
    // Note: If comrak's strict parser rejects it due to spaces, it will output:
    // <p>Inline math: $ E = mc^2 $ should work.</p> (i.e. no special span/element)

    assert!(
        output.html.contains("data-math-style") || output.html.contains("<math"),
        "Bug 3: Inline math WITH spaces was NOT recognized! HTML:\n{}",
        output.html
    );
}

/// RED test for Bug 2: Footnote definitions must appear at the end of the document
#[test]
fn html_export_must_move_footnotes_to_end() {
    let source = r#"
# Heading
Reference [^1].

[^1]: My definition here.

## Subheading
End of doc.
"#;
    let output = MarkdownRenderOps::render_with_katana_renderer(source).unwrap();

    let html = output.html;
    let def_pos = html.find("My definition here").expect("Footnote missing");
    let text_pos = html.find("End of doc").expect("Text missing");

    assert!(
        def_pos > text_pos,
        "Bug 2: Footnote definition should be placed after all document content, but instead was at pos {}, End of doc was at pos {}",
        def_pos,
        text_pos
    );
}

/// RED test for Bug 1: Unfenced Diagram Markers (e.g. <mxGraphModel> and @startuml)
/// must be properly rendered by the HTML export pipeline, not skipped.
#[test]
fn html_export_must_process_naked_diagram_markers() {
    let source = r#"
Here is a raw diagram:
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Test" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="10" y="10" width="80" height="40" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>

And naked PlantUML:
@startuml
A -> B: test
@enduml
"#;

    let output = MarkdownRenderOps::render(source, &DummyRenderer).unwrap();

    // We expect the diagram to be transformed by DummyRenderer, which outputs "Dummy Diagram"
    let dummy_counts = output.html.matches("Dummy Diagram").count();
    assert_eq!(
        dummy_counts, 2,
        "Bug 1: Naked <mxGraphModel> or @startuml blocks were skipped by HTML export! Output was: \n{}",
        output.html
    );
}

#[test]
fn pdf_export_writes_native_pdf_without_chromium() {
    assert!(PdfExporter.is_available());
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.pdf");

    do_pdf_export(sample_export_html(), &output);

    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.ends_with(b"%%EOF\n"));
    assert!(bytes.len() > 1024);
}

#[test]
fn pdf_export_splits_long_document_into_pages() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.pdf");

    do_pdf_export(&long_export_html(), &output);

    let bytes = std::fs::read(&output).unwrap();
    let page_count = pdf_page_count(&bytes);
    assert!(
        page_count > 1,
        "long native PDF export must keep paginated output instead of one tall page"
    );
}

#[test]
fn image_export_writes_native_png_without_chromium() {
    assert!(ImageExporter.is_available());
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(sample_export_html(), &output);

    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
    assert!(bytes.len() > 1024);
}

#[test]
fn image_export_keeps_inline_svg_diagram_without_chromium() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.png");

    do_image_export(inline_svg_export_html(), &output);

    let image = image::open(&output).unwrap().to_rgba8();
    assert!(
        image
            .pixels()
            .any(|pixel| pixel[2] > 180 && pixel[0] < 80 && pixel[1] < 160),
        "native image export must keep the inline SVG diagram pixels"
    );
}

#[test]
fn image_export_writes_native_jpeg_without_chromium() {
    assert!(ImageExporter.is_available());
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("document.jpeg");

    do_image_export(sample_export_html(), &output);

    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.starts_with(&[0xFF, 0xD8, 0xFF]));
    assert!(bytes.ends_with(&[0xFF, 0xD9]));
    assert!(bytes.len() > 1024);
}

#[test]
fn sample_mermaid_exports_html_pdf_png_and_jpeg_without_chromium() {
    let _guard = RENDER_ENV_LOCK.lock().unwrap();
    if katana_core::markdown::mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }
    let output = MarkdownRenderOps::render_with_katana_renderer(include_str!(
        "../../../assets/fixtures/sample_mermaid.md"
    ))
    .unwrap();
    let raw_mermaid_count = output.html.matches("language-mermaid").count();
    assert_eq!(raw_mermaid_count, 2);
    assert!(output.html.contains("zenuml"));
    assert!(!output.html.contains("katana-diagram-error"));
    assert!(
        output.html.matches("<svg").count() >= 26,
        "sample export HTML must include the 26 Mermaid SVG diagrams"
    );
    let dir = tempfile::tempdir().unwrap();

    do_pdf_export(&output.html, &dir.path().join("sample.pdf"));
    do_image_export(&output.html, &dir.path().join("sample.png"));
    do_image_export(&output.html, &dir.path().join("sample.jpeg"));

    assert!(
        std::fs::metadata(dir.path().join("sample.pdf"))
            .unwrap()
            .len()
            > 1024
    );
    assert!(
        std::fs::metadata(dir.path().join("sample.png"))
            .unwrap()
            .len()
            > 1024
    );
    assert!(
        std::fs::metadata(dir.path().join("sample.jpeg"))
            .unwrap()
            .len()
            > 1024
    );
}

#[test]
fn html_export_uses_unique_mermaid_svg_ids_for_multiple_diagrams() {
    let _guard = RENDER_ENV_LOCK.lock().unwrap();
    if katana_core::markdown::mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }
    let output = MarkdownRenderOps::render_with_katana_renderer(
        r#"
```mermaid
graph TD; A-->B
```

```mermaid
graph TD; C-->D
```
"#,
    )
    .unwrap();

    let ids = inline_svg_ids(&output.html);
    assert_eq!(ids.len(), 2);
    assert_unique_inline_svg_ids(&ids);
}

#[test]
fn mermaid_fixture_html_export_uses_unique_inline_svg_ids() {
    let _guard = RENDER_ENV_LOCK.lock().unwrap();
    if katana_core::markdown::mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }
    let output = MarkdownRenderOps::render_with_katana_renderer(include_str!(
        "../../../assets/fixtures/sample_mermaid.md"
    ))
    .unwrap();

    let ids = inline_svg_ids(&output.html);
    assert!(
        ids.len() >= 28,
        "assets/fixtures/sample_mermaid.md should render the supported Mermaid diagrams as inline SVG"
    );
    assert_unique_inline_svg_ids(&ids);
}

fn inline_svg_ids(html: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut position = 0;
    while let Some(start_offset) = html[position..].find("<svg") {
        let start = position + start_offset;
        let Some(open_end_offset) = html[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_offset;
        if let Some(id) = inline_svg_id(&html[start..open_end]) {
            ids.push(id);
        }
        position = open_end;
    }
    ids
}

fn inline_svg_id(open_tag: &str) -> Option<String> {
    let marker = r#"id=""#;
    let start = open_tag.find(marker)? + marker.len();
    let end = start + open_tag[start..].find('"')?;
    Some(open_tag[start..end].to_string())
}

fn assert_unique_inline_svg_ids(ids: &[String]) {
    let mut seen = HashSet::new();
    for id in ids {
        assert!(
            seen.insert(id),
            "HTML inline SVG must not duplicate Mermaid id `{id}` because marker and style references collide"
        );
    }
}

fn pdf_page_count(bytes: &[u8]) -> usize {
    String::from_utf8_lossy(bytes)
        .matches("/Type /Page /Parent")
        .count()
}

fn sample_export_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<body>
<h1>Export Sample</h1>
<p>Generated HTML is converted without Chrome or Chromium.</p>
<div class="katana-diagram mermaid">
<svg xmlns="http://www.w3.org/2000/svg" width="320" height="120">
<text x="20" y="40">Diagram label</text>
</svg>
</div>
</body>
</html>"#
}

fn long_export_html() -> String {
    let mut body = String::new();
    for index in 1..=120 {
        body.push_str(&format!(
            "<p>PDF pagination regression line {index}: exported content stays readable.</p>\n"
        ));
    }
    format!("<!DOCTYPE html><html><body>{body}</body></html>")
}

fn inline_svg_export_html() -> &'static str {
    r##"<!DOCTYPE html>
<html>
<body>
<h1>Diagram Export</h1>
<svg xmlns="http://www.w3.org/2000/svg" width="160" height="96" viewBox="0 0 160 96">
<rect x="8" y="8" width="144" height="80" fill="#0044ff"/>
<text x="24" y="54" fill="#ffffff">Diagram label</text>
</svg>
</body>
</html>"##
}
