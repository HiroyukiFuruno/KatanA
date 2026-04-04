use super::*;

#[test]
fn inject_theme_inserts_after_startuml() {
    let source = "@startuml\nA -> B\n@enduml";
    let result = inject_theme(source, &DiagramColorPreset::dark());
    assert!(result.starts_with("@startuml\n"));
    assert!(result.contains("skinparam backgroundColor transparent"));
    assert!(result.contains("skinparam defaultFontColor #E0E0E0"));
}

#[test]
fn inject_theme_wraps_when_no_startuml() {
    let source = "A -> B";
    let result = inject_theme(source, &DiagramColorPreset::dark());
    assert!(result.starts_with("@startuml\n"));
}
