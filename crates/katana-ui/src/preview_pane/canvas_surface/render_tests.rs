use std::sync::Arc;

use eframe::egui;
use katana_core::canvas::CanvasDocument;

use super::{CanvasLoadError, CanvasSurface, CanvasSurfaceLabels, cards::CanvasCards};

#[path = "render_regression_tests.rs"]
mod render_regression_tests;

pub(super) const FIXTURE: &str =
    include_str!("../../../../../assets/fixtures/katana/read-only-preview.canvas");
const TEST_SCREEN_WIDTH: f32 = 800.0;
const TEST_SCREEN_HEIGHT: f32 = 600.0;

pub(super) struct CanvasTest;

impl CanvasTest {
    pub(super) fn labels() -> CanvasSurfaceLabels<'static> {
        CanvasSurfaceLabels {
            fit: "\u{5168}\u{4f53}\u{8868}\u{793a}",
            empty: "\u{7a7a}\u{306e} Canvas",
            unsupported_node: "\u{672a}\u{5bfe}\u{5fdc}\u{30ce}\u{30fc}\u{30c9}",
            unsupported_fields: "\u{672a}\u{5bfe}\u{5fdc}\u{5c5e}\u{6027}",
            unsupported_link: "\u{30ea}\u{30f3}\u{30af}\u{672a}\u{5bfe}\u{5fdc}",
            unsupported_background: "\u{80cc}\u{666f}\u{672a}\u{5bfe}\u{5fdc}",
            load_error: "Canvas \u{8aad}\u{307f}\u{8fbc}\u{307f}\u{5931}\u{6557}",
            loading: "\u{30ad}\u{30e3}\u{30f3}\u{30d0}\u{30b9}\u{3092}\u{8aad}\u{307f}\u{8fbc}\u{307f}\u{4e2d}\u{2026}",
        }
    }

    pub(super) fn surface() -> CanvasSurface {
        let mut surface = CanvasSurface::default();
        surface.set_document(Arc::new(CanvasDocument::parse(FIXTURE).expect("fixture")));
        surface
    }

    pub(super) fn frame(
        surface: &mut CanvasSurface,
        context: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(TEST_SCREEN_WIDTH, TEST_SCREEN_HEIGHT),
            )),
            events,
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| surface.show(ui, &Self::labels()));
        output.textures_delta.clear();
        output
    }

    fn text(shape: &egui::Shape, result: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => result.push(text.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| Self::text(shape, result)),
            _ => {}
        }
    }

    pub(super) fn texts(output: &egui::FullOutput) -> Vec<String> {
        let mut result = Vec::new();
        for shape in &output.shapes {
            Self::text(&shape.shape, &mut result);
        }
        result
    }

    pub(super) fn pointer_button(pointer: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos: pointer,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
}

#[test]
fn real_egui_shapes_show_cards_labels_and_unsupported_reasons() {
    let mut surface = CanvasTest::surface();
    let output = CanvasTest::frame(&mut surface, &egui::Context::default(), vec![]);
    let texts = CanvasTest::texts(&output).join("\n");
    for expected in [
        "**Markdown**",
        "example.md",
        "Read-only Canvas",
        "references",
        "\u{30ea}\u{30f3}\u{30af}\u{672a}\u{5bfe}\u{5fdc}",
        "\u{672a}\u{5bfe}\u{5fdc}\u{30ce}\u{30fc}\u{30c9}",
        "\u{672a}\u{5bfe}\u{5fdc}\u{5c5e}\u{6027}",
    ] {
        assert!(texts.contains(expected), "missing: {expected}");
    }
    assert!(
        output
            .shapes
            .iter()
            .any(|shape| matches!(shape.shape, egui::Shape::Path(_)))
    );
}

#[test]
fn card_source_is_not_truncated_before_viewport_clipping() {
    let source = "x".repeat(100_000);
    let document = CanvasDocument::parse(&serde_json::json!({"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":40,"text":source}]}).to_string()).expect("long card");
    assert_eq!(
        CanvasCards::content(&document.nodes[0], &CanvasTest::labels()),
        source
    );
}

#[test]
fn typed_load_failure_is_visible_and_loading_can_restart() {
    let mut surface = CanvasSurface::default();
    let error = CanvasDocument::parse("not-json").expect_err("invalid Canvas input");
    surface.set_error(CanvasLoadError::InvalidCanvas(error));
    let context = egui::Context::default();
    let output = CanvasTest::frame(&mut surface, &context, vec![]);
    assert!(
        CanvasTest::texts(&output)
            .iter()
            .any(|text| text.contains("Canvas \u{8aad}\u{307f}\u{8fbc}\u{307f}\u{5931}\u{6557}"))
    );
    surface.loading();
    assert!(matches!(
        surface.state,
        super::state::CanvasLoadState::Loading
    ));
}

#[test]
fn error_from_core_parser_is_retained_as_typed_state() {
    let error = CanvasDocument::parse("{").expect_err("invalid JSON");
    let mut surface = CanvasSurface::default();
    surface.set_error(CanvasLoadError::InvalidCanvas(error));
    assert!(matches!(
        surface.state,
        super::state::CanvasLoadState::Failed(CanvasLoadError::InvalidCanvas(_))
    ));
}

#[test]
fn group_background_is_explicitly_marked_unsupported() {
    let document = CanvasDocument::parse(r#"{"nodes":[{"id":"g","type":"group","x":0,"y":0,"width":100,"height":80,"background":"missing.png"}]}"#).expect("group");
    assert!(
        CanvasCards::content(&document.nodes[0], &CanvasTest::labels())
            .contains("\u{80cc}\u{666f}\u{672a}\u{5bfe}\u{5fdc}")
    );
}
