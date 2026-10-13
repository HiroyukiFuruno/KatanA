use katana_core::markdown::types::DiagramKind;
use katana_core::preview::{PreviewFlattenOps, PreviewSection, PreviewSectionOps};

#[test]
fn test_regression_mermaid_after_list_recognition() {
    // Regression Case: A mermaid diagram immediately following a list item
    // Image 3 shows this might be the case where splitting fails.
    let source = "- List item\n\n```mermaid\ngraph TD;\nA-->B;\n```";

    // Step 1: Flattening (this is what PreviewPane does)
    let flattened = PreviewFlattenOps::flatten_list_code_blocks(source);

    // Step 2: Splitting
    let sections = PreviewSectionOps::split_sections(&flattened);

    // EXPECTATION: The second section should be a Diagram, not Markdown text.
    // If it's Markdown text, it will be rendered as a raw code block in UI.
    let diagram_found = sections.iter().any(|s| {
        matches!(
            s,
            PreviewSection::Diagram {
                kind: DiagramKind::Mermaid,
                ..
            }
        )
    });

    assert!(
        diagram_found,
        "Mermaid diagram WAS NOT recognized as a Diagram section. It remained as raw Markdown text. Sections: {:?}",
        sections
    );
}

#[test]
#[ignore = "RED: known limitation — trailing space after closing fence"]
fn test_regression_mermaid_trailing_space_red() {
    // Regression Case: Space after the closing fence
    // This is a common user error that should be handled gracefully but currently fails
    // because split_once("\n```") is too strict.
    let source = "```mermaid\ngraph TD;\n``` "; // Space here

    let sections = PreviewSectionOps::split_sections(source);

    let diagram_found = sections
        .iter()
        .any(|s| matches!(s, PreviewSection::Diagram { .. }));

    assert!(
        diagram_found,
        "Mermaid with trailing space after closing fence was NOT recognized. Sections: {:?}",
        sections
    );
}

#[test]
#[ignore = "RED: known limitation — no newline before opening fence"]
fn test_regression_mermaid_no_newline_before_fence_red() {
    // Regression Case: No newline before the opening fence (if it's not the start of string)
    let source = "Text before```mermaid\ngraph TD;\n```";

    let sections = PreviewSectionOps::split_sections(source);

    let diagram_found = sections
        .iter()
        .any(|s| matches!(s, PreviewSection::Diagram { .. }));

    assert!(
        diagram_found,
        "Mermaid without leading newline (inline-like) was NOT recognized. Sections: {:?}",
        sections
    );
}

#[test]
fn test_regression_indented_mermaid_recognition() {
    // Regression Case: Indented mermaid block (often found in nested lists)
    let source = "  ```mermaid\n  graph TD;\n  A-->B;\n  ```";

    let flattened = PreviewFlattenOps::flatten_list_code_blocks(source);
    let sections = PreviewSectionOps::split_sections(&flattened);

    let diagram_found = sections.iter().any(|s| {
        matches!(
            s,
            PreviewSection::Diagram {
                kind: DiagramKind::Mermaid,
                ..
            }
        )
    });

    assert!(
        diagram_found,
        "Indented Mermaid diagram was lost during flattening or splitting. Sections: {:?}",
        sections
    );
}

#[test]
fn test_regression_consecutive_diagram_recognition() {
    let source = r#"
# Consecutive
```mermaid
graph TD;
```
<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>
@startuml
A -> B
@enduml
"#;

    let sections = PreviewSectionOps::split_sections(source);

    let diagrams: Vec<_> = sections
        .iter()
        .filter(|s| matches!(s, PreviewSection::Diagram { .. }))
        .collect();

    assert_eq!(
        diagrams.len(),
        3,
        "Should recognize 3 consecutive diagrams. Found: {:?}",
        diagrams
    );
}
