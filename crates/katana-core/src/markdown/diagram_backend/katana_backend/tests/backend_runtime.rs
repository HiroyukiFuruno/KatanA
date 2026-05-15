use super::super::*;

const DRAWIO_SOURCE: &str = r#"<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Hello" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>"#;

fn render_input(
    language: DiagramBackendLanguage,
    source: impl Into<String>,
) -> DiagramBackendInput {
    DiagramBackendInput {
        language,
        source: source.into(),
        options: Default::default(),
        theme: super::super::super::types::DiagramThemeSnapshot::from_preset(
            "light",
            false,
            crate::markdown::color_preset::DiagramColorPreset::light(),
        ),
        document: super::super::super::types::DiagramDocumentContext::Detached {
            display_name: "diagram.md".to_string(),
        },
    }
}

#[test]
fn mermaid_backend_id_has_correct_language_and_implementation() {
    let backend = KatanaMermaidBackend;
    let id = backend.id();
    assert_eq!(id.language, DiagramBackendLanguage::Mermaid);
    assert_eq!(id.implementation, "kdr-mermaid");
}

#[test]
fn mermaid_backend_version_is_non_empty() {
    let version = KatanaMermaidBackend.version();
    assert!(version.value.contains("katana-diagram-renderer"));
    assert!(version.renderer_profile.contains("mermaid"));
}

#[test]
fn plantuml_backend_id_has_correct_language_and_implementation() {
    let backend = KatanaPlantUmlBackend;
    let id = backend.id();
    assert_eq!(id.language, DiagramBackendLanguage::PlantUml);
    assert_eq!(id.implementation, "katana-plantuml");
}

#[test]
fn plantuml_backend_version_is_non_empty() {
    assert!(!KatanaPlantUmlBackend.version().value.is_empty());
}

#[test]
fn drawio_backend_id_has_correct_language_and_implementation() {
    let backend = KatanaDrawIoBackend;
    let id = backend.id();
    assert_eq!(id.language, DiagramBackendLanguage::DrawIo);
    assert_eq!(id.implementation, "kdr-drawio");
}

#[test]
fn drawio_backend_version_is_non_empty() {
    let version = KatanaDrawIoBackend.version();
    assert!(version.value.contains("katana-diagram-renderer"));
    assert!(version.renderer_profile.contains("drawio"));
}

#[test]
fn drawio_backend_renders_svg_with_kdr_runtime() {
    let output = KatanaDrawIoBackend
        .render(&render_input(DiagramBackendLanguage::DrawIo, DRAWIO_SOURCE))
        .expect("Draw.io should render through kdr");

    match output {
        DiagramBackendOutput::HtmlFragment(svg) => assert!(svg.contains("<svg")),
        other => panic!("expected SVG fragment, got {other:?}"),
    }
}

#[test]
fn kdr_input_passes_full_light_theme_snapshot() {
    let input = DiagramBackendInput {
        language: DiagramBackendLanguage::Mermaid,
        source: "graph TD; A-->B".to_string(),
        options: Default::default(),
        theme: super::super::super::types::DiagramThemeSnapshot::from_preset(
            "light",
            false,
            crate::markdown::color_preset::DiagramColorPreset::light(),
        ),
        document: super::super::super::types::DiagramDocumentContext::Detached {
            display_name: "doc.md".to_string(),
        },
    };

    let kdr = kdr_input(katana_diagram_renderer::DiagramKind::Mermaid, &input);
    let theme = kdr.context.theme.expect("theme must be passed to kdr");

    assert_eq!(theme.mode, katana_diagram_renderer::RenderThemeMode::Light);
    assert_eq!(theme.text, "#333333");
    assert_eq!(theme.drawio_label_color, "#333333");
    assert_eq!(theme.mermaid_theme, "default");
    assert!(kdr.context.theme_fingerprint.is_some());
}
