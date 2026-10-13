use katana_core::markdown::fence::MarkdownFenceOps;
use katana_core::markdown::{
    DiagramBlock, DiagramKind, DiagramRenderer, DiagramResult, MarkdownRenderOps,
};

struct DummyRenderer;

impl DiagramRenderer for DummyRenderer {
    fn render(&self, block: &DiagramBlock) -> DiagramResult {
        let kind = match block.kind {
            DiagramKind::Mermaid => "mermaid",
            DiagramKind::PlantUml => "plantuml",
            DiagramKind::DrawIo => "drawio",
        };
        DiagramResult::Ok(format!(
            r#"<div class="katana-diagram" data-kind="{kind}"></div>"#
        ))
    }
}

#[test]
fn extract_tilde_mermaid_fence_block() {
    let result = MarkdownFenceOps::extract_fence_block("~~~mermaid\ngraph TD; A-->B\n~~~\nrest");

    let (block, rest) = result.expect("tilde mermaid fence should be extracted");
    assert_eq!(block.info, "mermaid");
    assert_eq!(block.content, "graph TD; A-->B");
    assert_eq!(rest, "rest");
}

#[test]
fn html_export_transforms_tilde_mermaid_fence() {
    let source = "before\n~~~mermaid\ngraph TD; A-->B\n~~~\nafter";
    let output = MarkdownRenderOps::render(source, &DummyRenderer).expect("render should pass");

    assert!(output.html.contains("katana-diagram"));
    assert!(output.html.contains(r#"data-kind="mermaid""#));
    assert!(!output.html.contains("language-mermaid"));
}

#[test]
fn html_export_transforms_tilde_plantuml_and_drawio_fences() {
    let source = "\
~~~plantuml
@startuml
A -> B
@enduml
~~~

~~~drawio
<mxGraphModel/>
~~~
";
    let output = MarkdownRenderOps::render(source, &DummyRenderer).expect("render should pass");

    assert!(!output.html.contains("language-plantuml"));
    assert!(!output.html.contains("language-drawio"));
    assert!(output.html.contains(r#"data-kind="plantuml""#));
    assert!(output.html.contains(r#"data-kind="drawio""#));
}
