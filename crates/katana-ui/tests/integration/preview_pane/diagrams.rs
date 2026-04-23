use egui_kittest::{Harness, kittest::Queryable};
use katana_ui::i18n::I18nOps;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const DRAWIO_SOURCE: &str = r#"<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Hello" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>"#;

fn diagram_md(lang: &str, body: &str) -> String {
    format!("# Diagram Test\n\n```{lang}\n{body}\n```\n\n## Footer\n")
}

fn assert_image(sections: &[RenderedSection], idx: usize, context: &str) {
    assert!(
        matches!(sections.get(idx), Some(RenderedSection::Image { .. })),
        "[{context}] Expected Image at index {idx}, got: {:?}",
        sections.get(idx)
    );
}

fn render_and_wait(lang: &str, source: &str) -> PreviewPane {
    let md = diagram_md(lang, source);
    let mut pane = PreviewPane::default();
    pane.full_render(
        &md,
        Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );
    pane.wait_for_renders();
    pane
}

fn build_harness(sections: Vec<RenderedSection>, width: f32, height: f32) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui, None, None, None, None);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();
    harness
}

fn assert_standard_diagram_markdown_visible(harness: &Harness) {
    let _heading = harness.get_by_label("Diagram Test");
    let _footer = harness.get_by_label("Footer");
}

#[test]
fn drawio_render_error_ui() {
    /* WHY: Verify that Draw.io rendering failures display a localized error message in the UI. */
    let _guard = crate::integration::get_serial_test_mutex().lock().unwrap();
    let sections = vec![
        RenderedSection::Markdown("# DrawIo Diagram\n".to_string(), 1),
        RenderedSection::Error {
            kind: "DrawIo".to_string(),
            _source: "<invalid/>".to_string(),
            message: "Failed to extract SVG from rendered HTML".to_string(),
            source_lines: 0,
        },
        RenderedSection::Markdown("## After diagram\n".to_string(), 1),
    ];
    let harness = build_harness(sections, 600.0, 300.0);
    let expected_error = I18nOps::tf(
        &I18nOps::get().error.render_error,
        &[
            ("kind", "DrawIo"),
            ("message", "Failed to extract SVG from rendered HTML"),
        ],
    );
    let _ = harness.get_by_label(&expected_error);
}

const MERMAID_SOURCE: &str = "graph TD\n    A[Start] --> B[End]";

#[test]
fn mermaid_both_states_render_semantically() {
    /* WHY: Verify Mermaid rendering transitions: 'CommandNotFound' when mmdc is missing, and 'Image' when available. */
    let _guard = crate::integration::get_serial_test_mutex().lock().unwrap();
    let saved_mmdc = std::env::var("MERMAID_MMDC").ok();

    unsafe { std::env::set_var("MERMAID_MMDC", "nonexistent_mmdc_for_idempotent_test") };

    let pane = render_and_wait("mermaid", MERMAID_SOURCE);
    assert!(
        matches!(pane.sections[1], RenderedSection::CommandNotFound { .. }),
        "With hidden mmdc, should be CommandNotFound, got: {:?}",
        pane.sections[1]
    );
    if let RenderedSection::CommandNotFound {
        tool_name,
        install_hint,
        ..
    } = &pane.sections[1]
    {
        assert!(tool_name.contains("mmdc"), "Tool name should mention mmdc");
        assert!(
            install_hint.contains("npm"),
            "Install hint should mention npm"
        );
        let expected = I18nOps::get()
            .error
            .missing_dependency
            .replace("{tool_name}", tool_name)
            .replace("{install_hint}", install_hint);
        let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
        assert_standard_diagram_markdown_visible(&harness);
        let _fallback = harness.get_by_label(&expected);
    }

    match &saved_mmdc {
        Some(v) => unsafe { std::env::set_var("MERMAID_MMDC", v) },
        None => unsafe { std::env::remove_var("MERMAID_MMDC") },
    }

    if katana_core::markdown::mermaid_renderer::MermaidRenderOps::is_mmdc_available() {
        let pane = render_and_wait("mermaid", MERMAID_SOURCE);
        assert!(
            matches!(
                pane.sections[1],
                RenderedSection::Image { .. } | RenderedSection::Error { .. }
            ),
            "[Mermaid rendered] Expected Image or Error at index 1, got: {:?}",
            pane.sections[1]
        );
        if let RenderedSection::Image { svg_data, alt, .. } = &pane.sections[1] {
            assert!(svg_data.width > 0, "Mermaid image width should be > 0");
            assert!(svg_data.height > 0, "Mermaid image height should be > 0");
            assert!(alt.contains("Mermaid"), "Alt should mention Mermaid");
        }
        let harness = build_harness(pane.sections.clone(), 600.0, 400.0);
        assert_standard_diagram_markdown_visible(&harness);
    }

    match saved_mmdc {
        Some(v) => unsafe { std::env::set_var("MERMAID_MMDC", v) },
        None => unsafe { std::env::remove_var("MERMAID_MMDC") },
    }
}

const PLANTUML_SOURCE: &str = "@startuml\nAlice -> Bob : Hello\n@enduml";

#[test]
fn plantuml_both_states_render_semantically() {
    /* WHY: Verify PlantUML rendering transitions: 'NotInstalled' when JAR is missing, and 'Image' when available. */
    let _guard = crate::integration::get_serial_test_mutex().lock().unwrap();
    let saved_jar = std::env::var("PLANTUML_JAR").ok();

    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/path/for/idempotent/test.jar") };

    let pane = render_and_wait("plantuml", PLANTUML_SOURCE);
    assert!(
        matches!(pane.sections[1], RenderedSection::NotInstalled { .. }),
        "With hidden jar, should be NotInstalled, got: {:?}",
        pane.sections[1]
    );
    if let RenderedSection::NotInstalled {
        kind, download_url, ..
    } = &pane.sections[1]
    {
        assert_eq!(kind, "PlantUML", "Kind should be PlantUML");
        assert!(
            download_url.contains("plantuml"),
            "URL should mention plantuml"
        );
        let tool_msg = I18nOps::tf(&I18nOps::get().tool.not_installed, &[("tool", kind)]);
        let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
        assert_standard_diagram_markdown_visible(&harness);
        let _fallback = harness.get_by_label(&tool_msg);
    }

    match &saved_jar {
        Some(v) => unsafe { std::env::set_var("PLANTUML_JAR", v) },
        None => unsafe { std::env::remove_var("PLANTUML_JAR") },
    }

    if katana_core::markdown::plantuml_renderer::PlantUmlRendererOps::find_plantuml_jar().is_some()
    {
        let pane = render_and_wait("plantuml", PLANTUML_SOURCE);
        assert!(
            matches!(
                pane.sections[1],
                RenderedSection::Image { .. } | RenderedSection::Error { .. }
            ),
            "[PlantUML rendered] Expected Image or Error at index 1, got: {:?}",
            pane.sections[1]
        );
        if let RenderedSection::Image { svg_data, alt, .. } = &pane.sections[1] {
            assert!(svg_data.width > 0, "PlantUML image width should be > 0");
            assert!(svg_data.height > 0, "PlantUML image height should be > 0");
            assert!(alt.contains("PlantUml"), "Alt should mention PlantUml");
        }
        let harness = build_harness(pane.sections.clone(), 600.0, 400.0);
        assert_standard_diagram_markdown_visible(&harness);
    }

    match saved_jar {
        Some(v) => unsafe { std::env::set_var("PLANTUML_JAR", v) },
        None => unsafe { std::env::remove_var("PLANTUML_JAR") },
    }
}

#[test]
fn mixed_diagram_document_renders_all_independently() {
    /* WHY: Verify that a document containing multiple mixed diagrams (Mermaid, Draw.io, PlantUML) renders all of them correctly in parallel. */
    let _guard = crate::integration::get_serial_test_mutex().lock().unwrap();
    let source = format!(
        "# Mixed\n\n```mermaid\n{MERMAID_SOURCE}\n```\n\n\
         ## DrawIo\n\n```drawio\n{DRAWIO_SOURCE}\n```\n\n\
         ## PlantUML\n\n```plantuml\n{PLANTUML_SOURCE}\n```\n\n\
         ## End\n"
    );
    let mut pane = PreviewPane::default();
    pane.full_render(
        &source,
        Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );

    assert_eq!(
        pane.sections.len(),
        7,
        "Expected 7 sections for mixed document"
    );

    pane.wait_for_renders();

    assert!(
        !pane
            .sections
            .iter()
            .any(|s| matches!(s, RenderedSection::Pending { .. })),
        "No Pending sections should remain after wait_for_renders"
    );

    assert!(
        matches!(
            pane.sections[1],
            RenderedSection::Image { .. }
                | RenderedSection::CommandNotFound { .. }
                | RenderedSection::Error { .. }
        ),
        "Mermaid should be Image or CommandNotFound, got: {:?}",
        pane.sections[1]
    );

    assert_image(&pane.sections, 3, "DrawIo in mixed document");

    assert!(
        matches!(
            pane.sections[5],
            RenderedSection::Image { .. }
                | RenderedSection::NotInstalled { .. }
                | RenderedSection::Error { .. }
        ),
        "PlantUML should be Image or NotInstalled, got: {:?}",
        pane.sections[5]
    );
}
