use std::sync::Arc;

use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use katana_core::canvas::CanvasDocument;

use super::{CanvasSurface, CanvasSurfaceLabels, CanvasTest};

struct RenderCase;

impl RenderCase {
    fn label_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
        let text = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == label => Some(text),
                _ => None,
            })
            .expect("actual edge label");
        text.galley.rect.translate(text.pos.to_vec2())
    }

    fn opaque_card(output: &egui::FullOutput, fill: egui::Color32) -> egui::Rect {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == fill => Some(rect.rect),
                _ => None,
            })
            .expect("actual opaque card fill")
    }
}

#[test]
fn loop_label_shapes_remain_outside_the_opaque_card() {
    let source = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":80,"text":"A"}],"edges":[{"id":"e","fromNode":"a","toNode":"a","label":"loop"}]}"#;
    let mut surface = CanvasSurface::default();
    surface.set_document(Arc::new(CanvasDocument::parse(source).expect("loop graph")));
    let context = egui::Context::default();
    let output = CanvasTest::frame(&mut surface, &context, vec![]);
    let label_rect = RenderCase::label_rect(&output, "loop");
    let fill = context.style_of(context.theme()).visuals.extreme_bg_color;
    let card = RenderCase::opaque_card(&output, fill);
    assert!(
        !card.intersects(label_rect),
        "loop label hidden by {card:?}"
    );
}

#[test]
fn painter_content_is_exposed_as_read_only_accessible_labels() {
    let mut surface = CanvasTest::surface();
    let harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(move |ui| surface.show(ui, &CanvasTest::labels()));
    for label in [
        "**Markdown**",
        "example.md",
        "Read-only Canvas",
        "references",
        "\u{30ea}\u{30f3}\u{30af}\u{672a}\u{5bfe}\u{5fdc}",
        "\u{672a}\u{5bfe}\u{5fdc}\u{30ce}\u{30fc}\u{30c9}",
    ] {
        assert!(harness.get_by_label_contains(label).rect().is_positive());
    }
}

#[test]
fn surface_labels_use_the_actual_canvas_translation_messages() {
    let messages = crate::i18n::I18nOps::get();
    let canvas = &messages.preview.canvas;
    let labels = CanvasSurfaceLabels::new(canvas);
    for (actual, expected) in [
        (labels.fit, canvas.fit.as_str()),
        (labels.empty, canvas.empty.as_str()),
        (labels.unsupported_node, canvas.unsupported_node.as_str()),
        (
            labels.unsupported_fields,
            canvas.unsupported_fields.as_str(),
        ),
        (labels.unsupported_link, canvas.unsupported_link.as_str()),
        (
            labels.unsupported_background,
            canvas.unsupported_background.as_str(),
        ),
        (labels.load_error, canvas.load_error.as_str()),
        (labels.loading, canvas.loading.as_str()),
    ] {
        assert_eq!(actual, expected);
    }
}
