use katana_core::markdown::{
    DiagramBlock, DiagramRenderer, DiagramResult, DiagramRuntimeAssetKind, DiagramRuntimeAssetOps,
    HtmlExporter, KatanaRenderer, MarkdownError, MarkdownRenderOps,
    color_preset::DiagramColorPreset,
};
use std::collections::HashSet;

fn mermaid_runtime_is_missing() -> bool {
    DiagramRuntimeAssetOps::find_path(DiagramRuntimeAssetKind::Mermaid).is_none()
}

struct SvgWithCssRenderer;

impl DiagramRenderer for SvgWithCssRenderer {
    fn render(&self, _block: &DiagramBlock) -> DiagramResult {
        DiagramResult::Ok(
            r##"<svg id="katana-mermaid-svg-test" xmlns="http://www.w3.org/2000/svg">
<style>#katana-mermaid-svg-test [id$="-arrowhead"] path{fill:#AAAAAA;stroke:#AAAAAA;}#katana-mermaid-svg-test [id$="-sequencenumber"]{fill:#AAAAAA;}</style>
<text x="0" y="16">Sequence</text>
</svg>"##
                .to_string(),
        )
    }
}

#[test]
fn rendered_svg_css_must_not_be_parsed_as_math() -> Result<(), MarkdownError> {
    let source = r#"
```mermaid
sequenceDiagram
```
"#;

    let output = MarkdownRenderOps::render(source, &SvgWithCssRenderer)?;

    assert!(
        output
            .html
            .contains(r##"[id$="-arrowhead"] path{fill:#AAAAAA;stroke:#AAAAAA;}"##),
        "SVG style CSS must remain raw CSS:\n{}",
        output.html
    );
    assert!(
        output
            .html
            .contains(r##"[id$="-sequencenumber"]{fill:#AAAAAA;}"##),
        "SVG style CSS must remain raw CSS:\n{}",
        output.html
    );
    assert!(
        !output.html.contains("data-math-style"),
        "Math parsing must not touch rendered diagram SVG HTML:\n{}",
        output.html
    );

    Ok(())
}

#[test]
fn repeated_same_mermaid_diagram_must_use_distinct_inline_svg_ids() -> Result<(), MarkdownError> {
    if mermaid_runtime_is_missing() {
        eprintln!("mermaid.min.js is not installed; skipping repeated-id regression");
        return Ok(());
    }
    let source = r#"
```mermaid
graph TD; A-->B
```

```mermaid
graph TD; A-->B
```
"#;

    let output = MarkdownRenderOps::render_with_katana_renderer(source)?;
    let ids = inline_svg_ids(&output.html);
    let unique_ids = ids.iter().collect::<HashSet<_>>();

    assert_eq!(
        ids.len(),
        2,
        "Expected two rendered Mermaid SVGs:\n{}",
        output.html
    );
    assert_eq!(
        unique_ids.len(),
        ids.len(),
        "Repeated Mermaid diagrams must not reuse inline SVG ids:\n{}",
        output.html
    );

    Ok(())
}

#[test]
fn mermaid_sequence_svg_css_must_not_be_parsed_as_math() -> Result<(), MarkdownError> {
    if mermaid_runtime_is_missing() {
        eprintln!("mermaid.min.js is not installed; skipping sequence CSS regression");
        return Ok(());
    }
    let source = r#"
```mermaid
sequenceDiagram
    participant User
    participant KatanA
    participant FileSystem
    User->>KatanA: Open file
    KatanA->>FileSystem: ReadMarkdown text
    FileSystem-->>KatanA: Render preview
```
"#;

    let output = MarkdownRenderOps::render_with_katana_renderer(source)?;

    assert!(
        output.html.contains(r#"[id$="-arrowhead"]"#),
        "Mermaid sequence CSS suffix selector must remain intact:\n{}",
        output.html
    );
    assert!(
        !output.html.contains("data-math-style"),
        "Math parsing must not touch Mermaid sequence SVG CSS:\n{}",
        output.html
    );

    Ok(())
}

#[test]
fn sample_diagrams_html_export_must_not_leak_svg_css_as_math() -> Result<(), MarkdownError> {
    if mermaid_runtime_is_missing() {
        eprintln!("mermaid.min.js is not installed; skipping sample HTML export regression");
        return Ok(());
    }
    let html = HtmlExporter::export(
        include_str!("../../../assets/fixtures/sample_diagrams.md"),
        &KatanaRenderer,
        DiagramColorPreset::dark(),
        None,
    )?;

    assert!(
        html.contains(r#"[id$="-arrowhead"]"#),
        "Sequence diagram SVG CSS suffix selector must remain intact:\n{}",
        html
    );
    assert!(
        inline_svg_fragments(&html)
            .iter()
            .all(|fragment| !fragment.contains("data-math-style")),
        "HTML export must not parse rendered SVG CSS as Markdown math:\n{}",
        html
    );
    let mermaid_ids = inline_svg_ids(&html)
        .into_iter()
        .filter(|id| id.starts_with("katana-mermaid-svg-"))
        .collect::<Vec<_>>();
    let unique_mermaid_ids = mermaid_ids.iter().collect::<HashSet<_>>();
    assert_eq!(
        unique_mermaid_ids.len(),
        mermaid_ids.len(),
        "HTML export must not duplicate inline Mermaid SVG ids:\n{}",
        html
    );

    Ok(())
}

fn inline_svg_ids(html: &str) -> Vec<String> {
    inline_svg_fragments(html)
        .iter()
        .filter_map(|fragment| inline_svg_id(fragment))
        .collect()
}

fn inline_svg_id(fragment: &str) -> Option<String> {
    let open_end = fragment.find('>')?;
    let marker = r#"id=""#;
    let start = fragment[..open_end].find(marker)? + marker.len();
    let end = start + fragment[start..open_end].find('"')?;
    Some(fragment[start..end].to_string())
}

fn inline_svg_fragments(html: &str) -> Vec<&str> {
    let mut fragments = Vec::new();
    let mut position = 0;
    while let Some(start_offset) = html[position..].find("<svg") {
        let start = position + start_offset;
        let Some(end_offset) = html[start..].find("</svg>") else {
            break;
        };
        let end = start + end_offset + "</svg>".len();
        fragments.push(&html[start..end]);
        position = end;
    }
    fragments
}
