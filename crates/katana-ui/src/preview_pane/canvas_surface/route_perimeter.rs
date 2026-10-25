use eframe::egui::{self, Pos2, Rect, Vec2};
use katana_core::canvas::EdgeSide;

pub(super) const SIDES: [EdgeSide; 4] = [
    EdgeSide::Top,
    EdgeSide::Right,
    EdgeSide::Bottom,
    EdgeSide::Left,
];
const TOP_INDEX: usize = 0;
const RIGHT_INDEX: usize = 1;
const BOTTOM_INDEX: usize = 2;
const LEFT_INDEX: usize = 3;

pub(super) struct OuterRoute;

impl OuterRoute {
    pub(super) fn side_index(side: EdgeSide) -> usize {
        match side {
            EdgeSide::Top => TOP_INDEX,
            EdgeSide::Right => RIGHT_INDEX,
            EdgeSide::Bottom => BOTTOM_INDEX,
            EdgeSide::Left => LEFT_INDEX,
        }
    }

    pub(super) fn normal(side: EdgeSide) -> Vec2 {
        match side {
            EdgeSide::Top => egui::vec2(0.0, -1.0),
            EdgeSide::Right => egui::vec2(1.0, 0.0),
            EdgeSide::Bottom => egui::vec2(0.0, 1.0),
            EdgeSide::Left => egui::vec2(-1.0, 0.0),
        }
    }

    pub(super) fn project(rect: Rect, anchor: Pos2, side: EdgeSide) -> Pos2 {
        match side {
            EdgeSide::Top => egui::pos2(anchor.x, rect.top()),
            EdgeSide::Right => egui::pos2(rect.right(), anchor.y),
            EdgeSide::Bottom => egui::pos2(anchor.x, rect.bottom()),
            EdgeSide::Left => egui::pos2(rect.left(), anchor.y),
        }
    }

    pub(super) fn paths(
        rect: Rect,
        from: (usize, Pos2),
        to: (usize, Pos2),
        full: bool,
    ) -> [Vec<Pos2>; 2] {
        [
            Self::walk(rect, from, to, true, full),
            Self::walk(rect, from, to, false, full),
        ]
    }

    fn walk(
        rect: Rect,
        from: (usize, Pos2),
        to: (usize, Pos2),
        clockwise: bool,
        full: bool,
    ) -> Vec<Pos2> {
        let count = SIDES.len();
        let mut steps = if clockwise {
            (to.0 + count - from.0) % count
        } else {
            (from.0 + count - to.0) % count
        };
        if full && steps == 0 {
            steps = count;
        }
        let mut points = vec![from.1];
        let mut side = from.0;
        for _ in 0..steps {
            points.push(Self::corner(rect, side, clockwise));
            side = (side + if clockwise { 1 } else { count - 1 }) % count;
        }
        points.push(to.1);
        points
    }

    fn corner(rect: Rect, side: usize, clockwise: bool) -> Pos2 {
        let next = (side + 1) % SIDES.len();
        let index = if clockwise { next } else { side };
        [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ][index]
    }
}
