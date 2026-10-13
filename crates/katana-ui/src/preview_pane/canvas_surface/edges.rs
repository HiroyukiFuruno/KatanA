use eframe::egui::{self, Pos2, Stroke, Vec2};
use katana_core::canvas::{CanvasEdge, EdgeEnd};

use super::{cards::CanvasLabels, colors::CanvasColors, geometry::CanvasLayout};

const EDGE_WIDTH: f32 = 1.5;
const ARROW_LENGTH: f32 = 10.0;
const ARROW_HALF_WIDTH: f32 = 4.0;
const LABEL_PADDING: f32 = 4.0;

pub(super) struct CanvasEdges;

impl CanvasEdges {
    pub(super) fn draw_ui(ui: &egui::Ui, edge: &CanvasEdge, layout: &CanvasLayout) {
        let Some(points) = layout.edge_points(edge) else {
            return;
        };
        let stroke = Stroke::new(
            EDGE_WIDTH,
            CanvasColors::resolve(edge.color.as_ref(), ui.visuals().text_color()),
        );
        Self::line_ui(ui, edge, &points, stroke);
        if let Some(label) = &edge.label {
            Self::label_ui(ui, edge, label, &points, layout, stroke.color);
        }
    }

    fn line_ui(ui: &egui::Ui, edge: &CanvasEdge, points: &[Pos2], stroke: Stroke) {
        let from = points[0];
        let to = points[points.len() - 1];
        let extent = points.iter().fold(egui::Rect::NOTHING, |rect, point| {
            rect.union(egui::Rect::from_pos(*point))
        });
        let visible = CanvasLayout::edge_visible(ui.clip_rect(), from, to, ARROW_LENGTH)
            || ui.clip_rect().intersects(extent.expand(ARROW_LENGTH));
        if !visible {
            return;
        }
        ui.painter().add(egui::Shape::line(points.to_vec(), stroke));
        if edge.from_end == EdgeEnd::Arrow {
            Self::arrow_ui(ui, from, points[1], stroke);
        }
        if edge.to_end == EdgeEnd::Arrow {
            Self::arrow_ui(ui, to, points[points.len() - 2], stroke);
        }
    }

    fn arrow_ui(ui: &egui::Ui, tip: Pos2, previous: Pos2, stroke: Stroke) {
        let direction = (tip - previous).normalized();
        if !direction.is_finite() || direction == Vec2::ZERO {
            return;
        }
        let base = tip - direction * ARROW_LENGTH;
        let normal = egui::vec2(-direction.y, direction.x) * ARROW_HALF_WIDTH;
        ui.painter().add(egui::Shape::line(
            vec![base + normal, tip, base - normal],
            stroke,
        ));
    }

    fn label_ui(
        ui: &egui::Ui,
        edge: &CanvasEdge,
        label: &str,
        points: &[Pos2],
        layout: &CanvasLayout,
        color: egui::Color32,
    ) {
        let galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            egui::TextStyle::Body.resolve(ui.style()),
            color,
        );
        let size = galley.size() + Vec2::splat(LABEL_PADDING * 2.0);
        let Some(center) = layout.edge_label_center(edge, points, size) else {
            return;
        };
        let rect = egui::Rect::from_center_size(center, galley.size());
        CanvasLabels::register_ui(ui, ui.id().with(("canvas-edge", &edge.id)), rect, label);
        ui.painter()
            .rect_filled(rect.expand(LABEL_PADDING), 0.0, ui.visuals().panel_fill);
        ui.painter().galley(rect.min, galley, color);
    }
}
