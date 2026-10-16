use std::collections::HashMap;

use eframe::egui::{self, Pos2, Rect, Vec2};
use katana_core::canvas::{CanvasDocument, CanvasEdge, CanvasGeometry, EdgeSide};

use super::CanvasLoadError;

#[path = "routing.rs"]
mod routing;

use routing::CanvasRoute;

pub(super) const MIN_ZOOM: f32 = 1.0e-16;
pub(super) const MAX_ZOOM: f32 = 16.0;
const EMPTY_SCENE_SIZE: f32 = 100.0;
const FIT_PADDING_FACTOR: f32 = 1.1;
const SCENE_MARGIN: f32 = 32.0;

#[derive(Default)]
pub(super) struct CanvasLayout {
    pub rects: Vec<Rect>,
    pub node_indices: HashMap<String, usize>,
    pub bounds: Option<Rect>,
}

impl CanvasLayout {
    pub(super) fn new(document: &CanvasDocument) -> Result<Self, CanvasLoadError> {
        let mut layout = Self::default();
        let origin = Self::relative_origin(document);
        for (index, node) in document.nodes.iter().enumerate() {
            let rect = Self::node_rect(&node.geometry, origin);
            if !Self::valid_rect(rect) {
                return Err(CanvasLoadError::UnrepresentableGeometry { index });
            }
            layout.bounds = Some(layout.bounds.map_or(rect, |bounds| bounds.union(rect)));
            layout.rects.push(rect);
            layout.node_indices.insert(node.id.clone(), index);
        }
        Ok(layout)
    }

    fn relative_origin(document: &CanvasDocument) -> (f64, f64) {
        let Some(first) = document.nodes.first() else {
            return (0.0, 0.0);
        };
        document
            .nodes
            .iter()
            .fold((first.geometry.x, first.geometry.y), |origin, node| {
                (origin.0.min(node.geometry.x), origin.1.min(node.geometry.y))
            })
    }

    pub(super) fn node_rect(geometry: &CanvasGeometry, origin: (f64, f64)) -> Rect {
        /* WHY: 共通原点を高精度のまま差し引き、単なる平行移動でカード幅を失わない。 */
        Rect::from_min_size(
            egui::pos2(
                (geometry.x - origin.0) as f32,
                (geometry.y - origin.1) as f32,
            ),
            egui::vec2(geometry.width as f32, geometry.height as f32),
        )
    }

    pub(super) fn valid_rect(rect: Rect) -> bool {
        rect.is_finite() && rect.is_positive()
    }

    pub(super) fn empty_bounds() -> Rect {
        Rect::from_center_size(Pos2::ZERO, Vec2::splat(EMPTY_SCENE_SIZE))
    }

    pub(super) fn fit_bounds(&self, viewport: Vec2) -> Rect {
        let bounds = self
            .bounds
            .unwrap_or_else(Self::empty_bounds)
            .expand(SCENE_MARGIN);
        if !viewport.is_finite() || viewport.min_elem() <= 0.0 {
            return bounds;
        }
        let zoom = (viewport / (bounds.size() * FIT_PADDING_FACTOR))
            .min_elem()
            .clamp(MIN_ZOOM, MAX_ZOOM);
        Rect::from_center_size(bounds.center(), viewport / zoom)
    }

    pub(super) fn zoom_view(rect: Rect, viewport: Vec2, factor: f32) -> Rect {
        if !Self::valid_rect(rect)
            || !viewport.is_finite()
            || viewport.min_elem() <= 0.0
            || !factor.is_finite()
            || factor <= 0.0
        {
            return rect;
        }
        let current = (viewport / rect.size()).min_elem();
        let target = (current * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        Rect::from_center_size(rect.center(), rect.size() * (current / target))
    }

    pub(super) fn anchor(rect: Rect, side: Option<EdgeSide>, toward: Pos2) -> Pos2 {
        let side = side.unwrap_or_else(|| Self::facing_side(rect.center(), toward));
        match side {
            EdgeSide::Top => rect.center_top(),
            EdgeSide::Right => rect.right_center(),
            EdgeSide::Bottom => rect.center_bottom(),
            EdgeSide::Left => rect.left_center(),
        }
    }

    fn facing_side(center: Pos2, toward: Pos2) -> EdgeSide {
        let delta = toward - center;
        if delta.x.abs() >= delta.y.abs() {
            if delta.x >= 0.0 {
                EdgeSide::Right
            } else {
                EdgeSide::Left
            }
        } else if delta.y >= 0.0 {
            EdgeSide::Bottom
        } else {
            EdgeSide::Top
        }
    }

    pub(super) fn edge_visible(clip: Rect, from: Pos2, to: Pos2, padding: f32) -> bool {
        clip.intersects(Rect::from_two_pos(from, to).expand(padding))
    }

    pub(super) fn edge_points(&self, edge: &CanvasEdge) -> Option<Vec<Pos2>> {
        let from_rect = *self.rects.get(*self.node_indices.get(&edge.from_node)?)?;
        let to_rect = *self.rects.get(*self.node_indices.get(&edge.to_node)?)?;
        CanvasRoute::points(from_rect, to_rect, edge)
    }

    pub(super) fn edge_label_center(
        &self,
        edge: &CanvasEdge,
        points: &[Pos2],
        size: Vec2,
    ) -> Option<Pos2> {
        let from = *self.rects.get(*self.node_indices.get(&edge.from_node)?)?;
        let to = *self.rects.get(*self.node_indices.get(&edge.to_node)?)?;
        Some(CanvasRoute::label_center(from, to, points, size))
    }
}
