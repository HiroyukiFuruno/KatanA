use super::*;

#[test]
fn test_diagram_kind_from_info() {
    assert_eq!(
        DiagramKind::from_info("mermaid"),
        Some(DiagramKind::Mermaid)
    );
    assert_eq!(
        DiagramKind::from_info("plantuml"),
        Some(DiagramKind::PlantUml)
    );
    assert_eq!(DiagramKind::from_info("drawio"), Some(DiagramKind::DrawIo));
    assert_eq!(DiagramKind::from_info("unknown"), None);
}

#[test]
fn test_diagram_kind_display_name() {
    assert_eq!(DiagramKind::Mermaid.display_name(), "Mermaid");
    assert_eq!(DiagramKind::PlantUml.display_name(), "PlantUML");
    assert_eq!(DiagramKind::DrawIo.display_name(), "Draw.io");
}

#[test]
fn mermaid_should_preserve_empty_and_zenuml_fences() {
    assert!(DiagramKind::Mermaid.should_preserve_fenced_source("  \n\t"));
    assert!(DiagramKind::Mermaid.should_preserve_fenced_source("zenuml\n    title Order Service"));
    assert!(
        DiagramKind::Mermaid
            .should_preserve_fenced_source("\u{feff}  ZenUML\n    title Order Service")
    );
    assert!(!DiagramKind::Mermaid.should_preserve_fenced_source("zenumlGraph\nA --> B"));
    assert!(!DiagramKind::Mermaid.should_preserve_fenced_source("graph TD\nA --> B"));
    assert!(!DiagramKind::DrawIo.should_preserve_fenced_source("zenuml\nA"));
}
