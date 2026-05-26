use super::super::*;
use crate::markdown::{DiagramBackendError, DiagramBackendOutput};

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
    assert_eq!(id.implementation, "kdv-kdr-mermaid");
}

#[test]
fn mermaid_backend_version_is_non_empty() {
    let version = KatanaMermaidBackend.version();
    assert!(version.value.contains("katana-document-viewer"));
    assert!(version.value.contains("katana-diagram-renderer"));
    assert!(version.renderer_profile.contains("mermaid"));
}

#[test]
fn plantuml_backend_id_has_correct_language_and_implementation() {
    let backend = KatanaPlantUmlBackend;
    let id = backend.id();
    assert_eq!(id.language, DiagramBackendLanguage::PlantUml);
    assert_eq!(id.implementation, "kdv-kdr-plantuml");
}

#[test]
fn plantuml_backend_version_is_non_empty() {
    let version = KatanaPlantUmlBackend.version();
    assert!(version.value.contains("katana-document-viewer"));
    assert!(version.value.contains("katana-diagram-renderer"));
    assert!(version.renderer_profile.contains("plantuml"));
}

#[test]
fn drawio_backend_id_has_correct_language_and_implementation() {
    let backend = KatanaDrawIoBackend;
    let id = backend.id();
    assert_eq!(id.language, DiagramBackendLanguage::DrawIo);
    assert_eq!(id.implementation, "kdv-kdr-drawio");
}

#[test]
fn drawio_backend_version_is_non_empty() {
    let version = KatanaDrawIoBackend.version();
    assert!(version.value.contains("katana-document-viewer"));
    assert!(version.value.contains("katana-diagram-renderer"));
    assert!(version.renderer_profile.contains("drawio"));
}

#[test]
fn drawio_backend_renders_svg_with_kdr_runtime() {
    let output = KatanaDrawIoBackend
        .render(&render_input(DiagramBackendLanguage::DrawIo, DRAWIO_SOURCE))
        .expect("Draw.io should render through kdv/kdr");

    match output {
        DiagramBackendOutput::HtmlFragment(svg) => assert!(svg.contains("<svg")),
        other => panic!("expected SVG fragment, got {other:?}"),
    }
}

#[test]
fn kdv_non_svg_output_is_not_treated_as_rendered_diagram() {
    let output = katana_document_viewer::RenderedDiagram {
        node_id: "node".to_string(),
        kind: "mermaid".to_string(),
        svg: "```mermaid\ngraph TD; A-->B\n```".to_string(),
    };

    assert!(matches!(
        kdv_diagram_output(&katana_markdown_model::DiagramKind::Mermaid, output),
        Err(DiagramBackendError::RenderFailed { .. })
    ));
}

#[test]
fn plantuml_non_svg_output_maps_to_not_installed() {
    let output = katana_document_viewer::RenderedDiagram {
        node_id: "node".to_string(),
        kind: "plantuml".to_string(),
        svg: "```plantuml\n@startuml\n@enduml\n```".to_string(),
    };

    assert!(matches!(
        kdv_diagram_output(&katana_markdown_model::DiagramKind::PlantUml, output),
        Err(DiagramBackendError::NotInstalled { .. })
    ));
}

#[test]
fn kdv_render_request_passes_full_light_theme_snapshot() {
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

    let theme = kdv_render_request_theme(katana_markdown_model::DiagramKind::Mermaid, &input);

    assert_eq!(theme.mode, katana_document_viewer::KdvThemeMode::Light);
    assert_eq!(theme.text, "#333333");
    assert_eq!(theme.diagram_text, "#333333");
    assert_eq!(theme.mermaid_theme, "default");
}

#[test]
fn plantuml_kdv_render_request_passes_full_light_theme_snapshot() {
    let input = render_input(
        DiagramBackendLanguage::PlantUml,
        "@startuml\nA -> B\n@enduml",
    );

    let theme = kdv_render_request_theme(katana_markdown_model::DiagramKind::PlantUml, &input);

    assert_eq!(theme.mode, katana_document_viewer::KdvThemeMode::Light);
    assert_eq!(theme.text, "#333333");
    assert_eq!(theme.diagram_fill, "#FEFECE");
    assert_eq!(theme.alert_background, "#FBFB77");
    assert_eq!(theme.diagram_text, "#333333");
}
