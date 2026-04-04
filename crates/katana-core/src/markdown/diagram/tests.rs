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
