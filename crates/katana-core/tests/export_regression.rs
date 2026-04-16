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
    let has_raw_code =
        output.html.contains("language-mermaid") || output.html.contains("<code>graph TD");

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
