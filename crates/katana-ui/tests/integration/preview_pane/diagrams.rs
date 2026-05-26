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

fn is_rendered_or_code_fallback(section: &RenderedSection, language: &str) -> bool {
    matches!(
        section,
        RenderedSection::Image { .. }
            | RenderedSection::NotInstalled { .. }
            | RenderedSection::Error { .. }
    ) || matches!(
        section,
        RenderedSection::Markdown(markdown, _)
            if markdown.contains("not supported")
                && markdown.contains(&format!("```{language}"))
    )
}

#[test]
fn drawio_render_error_ui() {
    /* WHY: Verify that Draw.io rendering failures display a localized error message in the UI. */
    let _guard = crate::integration::lock_serial_test_mutex();
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
    /* WHY: kdr owns Mermaid runtime assets now. Hiding legacy assets may still render successfully. */
    let _guard = crate::integration::lock_serial_test_mutex();
    let pane = crate::integration::test_helpers::MissingRendererAssetsOps::with(|| {
        render_and_wait("mermaid", MERMAID_SOURCE)
    });
    match &pane.sections[1] {
        RenderedSection::Image { svg_data, alt, .. } => {
            assert!(svg_data.width > 0, "Mermaid image width should be > 0");
            assert!(svg_data.height > 0, "Mermaid image height should be > 0");
            assert!(alt.contains("Mermaid"), "Alt should mention Mermaid");
            let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
            assert_standard_diagram_markdown_visible(&harness);
        }
        RenderedSection::NotInstalled { kind, message, .. } => {
            assert_eq!(kind, "Mermaid");
            assert!(!message.is_empty());
            let tool_msg = I18nOps::tf(
                &I18nOps::get().tool.not_installed,
                &[("tool", kind.as_str())],
            );
            let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
            assert_standard_diagram_markdown_visible(&harness);
            let _fallback = harness.get_by_label(&tool_msg);
        }
        _ => panic!("Expected Mermaid image or missing-runtime UI"),
    }
}

const PLANTUML_SOURCE: &str = "@startuml\nAlice -> Bob : Hello\n@enduml";

#[test]
fn plantuml_both_states_render_semantically() {
    /* WHY: KDV/KDR owns PlantUML runtime resolution, so UI semantics must be valid whether the renderer produces an image or reports a runtime error. */
    let _guard = crate::integration::lock_serial_test_mutex();
    let pane = render_and_wait("plantuml", PLANTUML_SOURCE);
    match &pane.sections[1] {
        RenderedSection::Image { svg_data, alt, .. } => {
            assert!(svg_data.width > 0, "PlantUML image width should be > 0");
            assert!(svg_data.height > 0, "PlantUML image height should be > 0");
            assert!(alt.contains("PlantUml"), "Alt should mention PlantUml");
            let harness = build_harness(pane.sections.clone(), 600.0, 400.0);
            assert_standard_diagram_markdown_visible(&harness);
        }
        RenderedSection::NotInstalled { kind, message, .. } => {
            assert_eq!(kind, "PlantUML", "Kind should be PlantUML");
            assert!(!message.is_empty());
            let tool_msg = I18nOps::tf(&I18nOps::get().tool.not_installed, &[("tool", kind)]);
            let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
            assert_standard_diagram_markdown_visible(&harness);
            let _fallback = harness.get_by_label(&tool_msg);
        }
        RenderedSection::Error { .. } => {
            let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
            assert_standard_diagram_markdown_visible(&harness);
        }
        _ => panic!("Expected PlantUML image, runtime message, or render error"),
    }
}

#[test]
fn mixed_diagram_document_renders_all_independently() {
    /* WHY: Verify mixed diagrams resolve independently without invoking real browser renderers. */
    let _guard = crate::integration::lock_serial_test_mutex();
    let source = format!(
        "# Mixed\n\n```mermaid\n{MERMAID_SOURCE}\n```\n\n\
         ## DrawIo\n\n```drawio\n{DRAWIO_SOURCE}\n```\n\n\
         ## PlantUML\n\n```plantuml\n{PLANTUML_SOURCE}\n```\n\n\
         ## End\n"
    );
    let mut pane = PreviewPane::default();
    crate::integration::test_helpers::MissingRendererAssetsOps::with(|| {
        pane.full_render(
            &source,
            Path::new("/tmp/test.md"),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            false,
            4,
        );
    });

    assert_eq!(
        pane.sections.len(),
        7,
        "Expected 7 sections for mixed document"
    );

    crate::integration::test_helpers::MissingRendererAssetsOps::with(|| pane.wait_for_renders());

    assert!(
        !pane
            .sections
            .iter()
            .any(|s| matches!(s, RenderedSection::Pending { .. })),
        "No Pending sections should remain after wait_for_renders"
    );

    assert!(
        is_rendered_or_code_fallback(&pane.sections[1], "mermaid"),
        "Mermaid should be Image, NotInstalled, Error, or code fallback, got: {:?}",
        pane.sections[1]
    );

    assert!(
        is_rendered_or_code_fallback(&pane.sections[3], "drawio"),
        "DrawIo should be Image, NotInstalled, Error, or code fallback, got: {:?}",
        pane.sections[3]
    );

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
