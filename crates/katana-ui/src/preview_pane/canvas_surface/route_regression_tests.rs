use eframe::egui::{self, Pos2, Rect, Vec2};
use katana_core::canvas::{
    CanvasDocument, CanvasEdge, CanvasGeometry, CanvasNode, EdgeEnd, EdgeSide, NodeContent,
};

use super::CanvasLayout;

const CARD_HEIGHT: f64 = 80.0;

const SIDES: [EdgeSide; 4] = [
    EdgeSide::Top,
    EdgeSide::Right,
    EdgeSide::Bottom,
    EdgeSide::Left,
];

struct RouteCase;

impl RouteCase {
    fn node(id: &str, position: (f64, f64)) -> CanvasNode {
        CanvasNode {
            id: id.into(),
            geometry: CanvasGeometry {
                x: position.0,
                y: position.1,
                width: 100.0,
                height: CARD_HEIGHT,
            },
            color: None,
            content: NodeContent::Text { text: id.into() },
        }
    }

    fn document(target: (f64, f64)) -> CanvasDocument {
        CanvasDocument {
            nodes: vec![Self::node("a", (0.0, 0.0)), Self::node("b", target)],
            ..Default::default()
        }
    }

    fn edge(from: EdgeSide, to: EdgeSide, same_node: bool) -> CanvasEdge {
        CanvasEdge {
            id: "edge".into(),
            from_node: "a".into(),
            from_side: Some(from),
            from_end: EdgeEnd::Arrow,
            to_node: if same_node { "a" } else { "b" }.into(),
            to_side: Some(to),
            to_end: EdgeEnd::Arrow,
            color: None,
            label: Some("loop".into()),
        }
    }

    fn normal(side: EdgeSide) -> Vec2 {
        match side {
            EdgeSide::Top => egui::vec2(0.0, -1.0),
            EdgeSide::Right => egui::vec2(1.0, 0.0),
            EdgeSide::Bottom => egui::vec2(0.0, 1.0),
            EdgeSide::Left => egui::vec2(-1.0, 0.0),
        }
    }

    fn assert_tangents(layout: &CanvasLayout, edge: &CanvasEdge) -> Vec<Pos2> {
        let points = layout.edge_points(edge).expect("existing endpoints");
        assert!(points.len() >= 2);
        assert!(points.iter().all(|point| point.is_finite()));
        let last = points.len() - 1;
        let from = Self::normal(edge.from_side.expect("explicit start side"));
        let to = Self::normal(edge.to_side.expect("explicit end side"));
        let first_segment = (points[1] - points[0]).normalized();
        let last_segment = (points[last - 1] - points[last]).normalized();
        assert_eq!(first_segment, from, "start side: {:?}", edge.from_side);
        assert_eq!(last_segment, to, "end side: {:?}", edge.to_side);
        points
    }

    fn crosses_interior(rect: Rect, start: Pos2, end: Pos2) -> bool {
        let extent = Rect::from_two_pos(start, end);
        if start.y == end.y {
            start.y > rect.top()
                && start.y < rect.bottom()
                && extent.left() < rect.right()
                && extent.right() > rect.left()
        } else if start.x == end.x {
            start.x > rect.left()
                && start.x < rect.right()
                && extent.top() < rect.bottom()
                && extent.bottom() > rect.top()
        } else {
            panic!("route must use orthogonal segments");
        }
    }

    fn assert_clear(layout: &CanvasLayout, edge: &CanvasEdge) {
        let points = Self::assert_tangents(layout, edge);
        for rect in &layout.rects {
            for pair in points.windows(2) {
                assert!(
                    !Self::crosses_interior(*rect, pair[0], pair[1]),
                    "card {rect:?} crossed by {pair:?}"
                );
            }
        }
    }
}

#[test]
fn every_explicit_edge_side_avoids_both_nonoverlapping_cards() {
    for target in [
        (300.0, 0.0),
        (-300.0, 0.0),
        (0.0, 240.0),
        (0.0, -240.0),
        (150.0, -60.0),
        (110.0, 40.0),
        (105.0, 0.0),
    ] {
        let layout = CanvasLayout::new(&RouteCase::document(target)).expect("layout");
        for from in SIDES {
            for to in SIDES {
                RouteCase::assert_clear(&layout, &RouteCase::edge(from, to, false));
            }
        }
    }
}

#[test]
fn every_self_loop_side_keeps_arrows_outside_its_card() {
    let mut document = RouteCase::document((300.0, 0.0));
    document.nodes.truncate(1);
    let layout = CanvasLayout::new(&document).expect("layout");
    for from in SIDES {
        for to in SIDES {
            RouteCase::assert_clear(&layout, &RouteCase::edge(from, to, true));
        }
    }
}

#[test]
fn overlapping_cards_keep_finite_routes_and_explicit_endpoint_tangents() {
    for target in [(40.0, 20.0), (0.0, 0.0)] {
        let layout = CanvasLayout::new(&RouteCase::document(target)).expect("layout");
        for from in SIDES {
            for to in SIDES {
                RouteCase::assert_tangents(&layout, &RouteCase::edge(from, to, false));
            }
        }
    }
}

#[test]
fn translated_graph_retains_its_relative_card_geometry() {
    let baseline = RouteCase::document((300.0, 0.0));
    let expected = CanvasLayout::new(&baseline).expect("baseline");
    let mut translated = baseline;
    for node in &mut translated.nodes {
        node.geometry.x += 4_294_967_296.0;
        node.geometry.y -= 4_294_967_296.0;
    }
    let actual = CanvasLayout::new(&translated).expect("same graph at a translated origin");
    assert_eq!(actual.rects, expected.rects);
    assert_eq!(actual.bounds, expected.bounds);
}
