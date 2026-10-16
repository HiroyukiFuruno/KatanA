use eframe::egui::{self, Pos2, Rect, Vec2};
use katana_core::canvas::{CanvasDocument, EdgeSide};

use super::{CanvasLoadError, geometry::CanvasLayout};

#[path = "route_regression_tests.rs"]
mod route_regression_tests;

const FIT_ASPECT_ROUNDING_ULPS: u32 = 1;

#[test]
fn empty_graph_fit_has_finite_positive_bounds() {
    let layout = CanvasLayout::new(&CanvasDocument::default()).expect("empty layout");
    assert!(CanvasLayout::valid_rect(
        layout.fit_bounds(egui::vec2(800.0, 600.0))
    ));
}

#[test]
fn fit_contains_negative_positioned_graph_and_preserves_aspect() {
    let mut layout = CanvasLayout::default();
    let bounds = Rect::from_min_max(egui::pos2(-200.0, -50.0), egui::pos2(300.0, 150.0));
    layout.bounds = Some(bounds);
    let fitted = layout.fit_bounds(egui::vec2(800.0, 600.0));
    assert!(fitted.contains_rect(bounds));
    let actual = fitted.aspect_ratio();
    let expected = 800.0_f32 / 600.0;
    let ulps = actual.to_bits().abs_diff(expected.to_bits());
    /* WHY: 除算と矩形端点の丸めで生じる1ULPを許容し、旧比較の境界一致排除を直す。 */
    eprintln!("fit aspect actual={actual:?} expected={expected:?} ULP difference={ulps}");
    assert!(ulps <= FIT_ASPECT_ROUNDING_ULPS);
}

#[test]
fn explicit_sides_select_exact_card_edge_midpoints() {
    let rect = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 80.0));
    let anchors: Vec<_> = [
        EdgeSide::Top,
        EdgeSide::Right,
        EdgeSide::Bottom,
        EdgeSide::Left,
    ]
    .into_iter()
    .map(|side| CanvasLayout::anchor(rect, Some(side), Pos2::ZERO))
    .collect();
    assert_eq!(
        anchors,
        [
            egui::pos2(50.0, 0.0),
            egui::pos2(100.0, 40.0),
            egui::pos2(50.0, 80.0),
            egui::pos2(0.0, 40.0)
        ]
    );
}

#[test]
fn omitted_side_faces_the_other_card() {
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
    assert_eq!(
        CanvasLayout::anchor(rect, None, egui::pos2(-200.0, 40.0)),
        rect.left_center()
    );
}

#[test]
fn viewport_culling_keeps_crossing_edges_and_rejects_distant_edges() {
    let clip = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
    assert!(CanvasLayout::edge_visible(
        clip,
        egui::pos2(-50.0, 50.0),
        egui::pos2(150.0, 50.0),
        1.0
    ));
    assert!(!CanvasLayout::edge_visible(
        clip,
        egui::pos2(500.0, 500.0),
        egui::pos2(600.0, 500.0),
        1.0
    ));
}

#[test]
fn invalid_zoom_inputs_do_not_poison_the_view() {
    let rect = CanvasLayout::empty_bounds();
    for factor in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
        assert_eq!(
            CanvasLayout::zoom_view(rect, Vec2::splat(100.0), factor),
            rect
        );
    }
}

#[test]
fn unrepresentable_card_dimensions_return_typed_error() {
    let source = r#"{"nodes":[{"id":"origin","type":"text","x":0,"y":0,"width":1,"height":1,"text":"origin"},{"id":"a","type":"text","x":9007199254740000,"y":0,"width":1,"height":1,"text":"a"}]}"#;
    let document = CanvasDocument::parse(source).expect("valid integer geometry");
    assert!(matches!(
        CanvasLayout::new(&document),
        Err(CanvasLoadError::UnrepresentableGeometry { index: 1 })
    ));
}
