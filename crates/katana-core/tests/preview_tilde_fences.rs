use katana_core::markdown::diagram::DiagramKind;
use katana_core::preview::{PreviewSection, PreviewSectionOps};

#[test]
fn tilde_mermaid_fence_is_split_into_diagram_section() {
    let source = "before\n~~~mermaid\ngraph TD; A-->B\n~~~\nafter";
    let sections = PreviewSectionOps::split_into_sections(source);

    assert_eq!(sections.len(), 3);
    assert!(matches!(
        sections[1],
        PreviewSection::Diagram {
            kind: DiagramKind::Mermaid,
            ..
        }
    ));
}

#[test]
fn tilde_plantuml_fence_is_split_into_diagram_section() {
    let source = "before\n~~~plantuml\n@startuml\nA -> B\n@enduml\n~~~\nafter";
    let sections = PreviewSectionOps::split_into_sections(source);

    assert_eq!(sections.len(), 3);
    assert!(matches!(
        sections[1],
        PreviewSection::Diagram {
            kind: DiagramKind::PlantUml,
            ..
        }
    ));
}

#[test]
fn tilde_drawio_fence_is_split_into_diagram_section() {
    let source = "before\n~~~drawio\n<mxGraphModel/>\n~~~\nafter";
    let sections = PreviewSectionOps::split_into_sections(source);

    assert_eq!(sections.len(), 3);
    assert!(matches!(
        sections[1],
        PreviewSection::Diagram {
            kind: DiagramKind::DrawIo,
            ..
        }
    ));
}

#[test]
fn nested_tilde_mermaid_inside_markdown_fence_stays_markdown() {
    let source = "\
~~~markdown
~~~mermaid
graph TD
    A --> B
~~~
~~~
";
    let sections = PreviewSectionOps::split_into_sections(source);

    assert_eq!(sections.len(), 1);
    assert!(matches!(sections[0], PreviewSection::Markdown(_, _)));
}
