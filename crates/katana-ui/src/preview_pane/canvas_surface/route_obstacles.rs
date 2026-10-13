use eframe::egui::{Pos2, Rect};
use katana_core::canvas::EdgeSide;

pub(super) struct RouteObstacles {
    from: Rect,
    to: Rect,
}

impl RouteObstacles {
    pub(super) fn new(from: Rect, to: Rect) -> Self {
        Self { from, to }
    }

    pub(super) fn overlaps(&self) -> bool {
        self.from.intersects(self.to)
    }

    pub(super) fn clear(&self, points: &[Pos2]) -> bool {
        points.windows(2).all(|pair| {
            !Self::crosses(self.from, pair[0], pair[1]) && !Self::crosses(self.to, pair[0], pair[1])
        })
    }

    fn crosses(rect: Rect, from: Pos2, to: Pos2) -> bool {
        let extent = Rect::from_two_pos(from, to);
        if from.y == to.y {
            from.y > rect.top()
                && from.y < rect.bottom()
                && extent.left() < rect.right()
                && extent.right() > rect.left()
        } else if from.x == to.x {
            from.x > rect.left()
                && from.x < rect.right()
                && extent.top() < rect.bottom()
                && extent.bottom() > rect.top()
        } else {
            true
        }
    }

    pub(super) fn clearance(anchor: Pos2, side: EdgeSide, other: Rect, requested: f32) -> f32 {
        let aligned = match side {
            EdgeSide::Top | EdgeSide::Bottom => anchor.x > other.left() && anchor.x < other.right(),
            EdgeSide::Left | EdgeSide::Right => anchor.y > other.top() && anchor.y < other.bottom(),
        };
        let distance = match side {
            EdgeSide::Top => anchor.y - other.bottom(),
            EdgeSide::Right => other.left() - anchor.x,
            EdgeSide::Bottom => other.top() - anchor.y,
            EdgeSide::Left => anchor.x - other.right(),
        };
        if aligned && distance > 0.0 {
            requested.min(distance / 2.0)
        } else {
            requested
        }
    }
}
