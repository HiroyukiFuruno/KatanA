use eframe::egui::{self, Pos2, Rect, Vec2};
use katana_core::canvas::{CanvasEdge, EdgeSide};

use super::CanvasLayout;

#[path = "route_obstacles.rs"]
mod obstacles;
#[path = "route_perimeter.rs"]
mod perimeter;

use obstacles::RouteObstacles;
use perimeter::{OuterRoute, SIDES};

const CARD_CLEARANCE: f32 = 24.0;

pub(super) struct CanvasRoute;

impl CanvasRoute {
    pub(super) fn points(from: Rect, to: Rect, edge: &CanvasEdge) -> Option<Vec<Pos2>> {
        let (from_side, to_side) = Self::sides(from, to, edge);
        let start = CanvasLayout::anchor(from, Some(from_side), to.center());
        let end = CanvasLayout::anchor(to, Some(to_side), from.center());
        let obstacles = RouteObstacles::new(from, to);
        let bounds = from.union(to).expand(CARD_CLEARANCE);
        let route = if obstacles.overlaps() {
            Self::overlapping(bounds, (from_side, start), (to_side, end))
        } else {
            let a = Self::lead(start, from_side, to);
            let b = Self::lead(end, to_side, from);
            Self::between(bounds, a, b, &obstacles)?
        };
        let mut points = vec![start];
        points.extend(route);
        points.push(end);
        points.dedup();
        Some(points)
    }

    fn sides(from: Rect, to: Rect, edge: &CanvasEdge) -> (EdgeSide, EdgeSide) {
        let same = edge.from_node == edge.to_node;
        let start = edge.from_side.unwrap_or_else(|| {
            if same {
                EdgeSide::Right
            } else {
                CanvasLayout::facing_side(from.center(), to.center())
            }
        });
        let end = edge.to_side.unwrap_or_else(|| {
            if same {
                EdgeSide::Top
            } else {
                CanvasLayout::facing_side(to.center(), from.center())
            }
        });
        (start, end)
    }

    fn lead(anchor: Pos2, side: EdgeSide, other: Rect) -> Pos2 {
        let distance = RouteObstacles::clearance(anchor, side, other, CARD_CLEARANCE);
        anchor + OuterRoute::normal(side) * distance
    }

    fn shortest(paths: impl IntoIterator<Item = Vec<Pos2>>) -> Option<Vec<Pos2>> {
        paths
            .into_iter()
            .min_by(|a, b| Self::length(a).total_cmp(&Self::length(b)))
    }

    fn length(points: &[Pos2]) -> f32 {
        points
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).length())
            .sum()
    }

    fn overlapping(bounds: Rect, from: (EdgeSide, Pos2), to: (EdgeSide, Pos2)) -> Vec<Pos2> {
        let from_index = OuterRoute::side_index(from.0);
        let to_index = OuterRoute::side_index(to.0);
        let a = (from_index, OuterRoute::project(bounds, from.1, from.0));
        let b = (to_index, OuterRoute::project(bounds, to.1, to.0));
        /* WHY: 重なるカードの接点は相手内部に入り得るため、自身の外向き接線を保って外周へ出る。 */
        let [clockwise, counterclockwise] = OuterRoute::paths(bounds, a, b, from.1 == to.1);
        if Self::length(&clockwise) <= Self::length(&counterclockwise) {
            clockwise
        } else {
            counterclockwise
        }
    }

    fn between(
        bounds: Rect,
        from: Pos2,
        to: Pos2,
        obstacles: &RouteObstacles,
    ) -> Option<Vec<Pos2>> {
        let simple = [
            vec![from, to],
            vec![from, egui::pos2(from.x, to.y), to],
            vec![from, egui::pos2(to.x, from.y), to],
        ];
        if let Some(path) = Self::shortest(simple.into_iter().filter(|path| obstacles.clear(path)))
        {
            return Some(path);
        }
        Self::outside(bounds, from, to, obstacles)
    }

    fn outside(
        bounds: Rect,
        from: Pos2,
        to: Pos2,
        obstacles: &RouteObstacles,
    ) -> Option<Vec<Pos2>> {
        let starts = Self::exits(bounds, from, obstacles);
        let ends = Self::exits(bounds, to, obstacles);
        let mut paths = Vec::new();
        for start in starts {
            for end in &ends {
                for path in OuterRoute::paths(bounds, start, *end, false) {
                    let mut candidate = vec![from];
                    candidate.extend(path);
                    candidate.push(to);
                    paths.push(candidate);
                }
            }
        }
        Self::shortest(paths)
    }

    fn exits(bounds: Rect, port: Pos2, obstacles: &RouteObstacles) -> Vec<(usize, Pos2)> {
        SIDES
            .iter()
            .enumerate()
            .filter_map(|(index, side)| {
                let exit = OuterRoute::project(bounds, port, *side);
                obstacles.clear(&[port, exit]).then_some((index, exit))
            })
            .collect()
    }

    pub(super) fn label_center(from: Rect, to: Rect, points: &[Pos2], size: Vec2) -> Pos2 {
        let candidate = points
            .windows(2)
            .filter_map(|pair| {
                let center = pair[0] + (pair[1] - pair[0]) / 2.0;
                let label = Rect::from_center_size(center, size);
                (!from.intersects(label) && !to.intersects(label))
                    .then_some((center, (pair[1] - pair[0]).length_sq()))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((center, _)) = candidate {
            return center;
        }
        let bounds = from.union(to);
        let top = points
            .iter()
            .fold(bounds.top(), |top, point| top.min(point.y));
        egui::pos2(bounds.center().x, top - size.y / 2.0)
    }
}
