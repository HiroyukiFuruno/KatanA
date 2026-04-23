use katana_core::markdown::{DiagramBlock, DiagramRenderer, DiagramResult, MarkdownRenderOps};

struct DummyRenderer;

impl DiagramRenderer for DummyRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        DiagramResult::Ok(format!(
            "<div class=\"katana-diagram\">Dummy Diagram {}</div>",
            block.source.trim()
        ))
    }
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
    let output = MarkdownRenderOps::render_with_katana_renderer(source).unwrap();

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
    let output = MarkdownRenderOps::render_with_katana_renderer(source).unwrap();

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
