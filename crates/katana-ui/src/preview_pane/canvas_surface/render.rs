use eframe::egui;
use katana_core::canvas::CanvasDocument;

use super::{CanvasSurfaceLabels, cards::CanvasCards, edges::CanvasEdges, geometry::CanvasLayout};

pub(super) struct CanvasRenderer;

impl CanvasRenderer {
    pub(super) fn draw_ui(
        ui: &egui::Ui,
        document: &CanvasDocument,
        layout: &CanvasLayout,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        for edge in &document.edges {
            CanvasEdges::draw_ui(ui, edge, layout);
        }
        for (node, rect) in document.nodes.iter().zip(&layout.rects) {
            if ui.clip_rect().intersects(*rect) {
                CanvasCards::draw_ui(ui, node, *rect, labels);
            }
        }
    }
}
