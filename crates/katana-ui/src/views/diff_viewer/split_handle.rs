use super::split::SplitLayout;
use eframe::egui;

const SPLIT_MIN_RATIO: f32 = 0.1_f32;
const SPLIT_MAX_RATIO: f32 = 0.9_f32;
const SPLITTER_WIDTH: f32 = 10.0_f32;
const SPLITTER_EDGE_INSET: f32 = 1.0_f32;
const SPLITTER_CENTER_WIDTH: f32 = 2.0_f32;
const SPLITTER_STROKE_WIDTH: f32 = 1.0_f32;

pub(super) struct DiffViewerSplitHandleOps;

impl DiffViewerSplitHandleOps {
    pub(super) const WIDTH: f32 = SPLITTER_WIDTH;

    pub(super) fn show(
        ui: &mut egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        layout: SplitLayout,
        ratio: f32,
    ) -> f32 {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(SPLITTER_WIDTH, layout.height),
            egui::Sense::hover(),
        );
        let id = egui::Id::new(("diff_splitter_handle", file.path.as_path()));
        let response = ui.interact(rect, id, egui::Sense::drag());
        Self::paint(ui, rect, &response);
        if response.dragged() {
            (ratio + response.drag_delta().x / layout.available_width)
                .clamp(SPLIT_MIN_RATIO, SPLIT_MAX_RATIO)
        } else {
            ratio
        }
    }

    fn paint(ui: &mut egui::Ui, rect: egui::Rect, response: &egui::Response) {
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        let fill = if response.hovered() || response.dragged() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        ui.painter().rect_filled(rect, 0.0, fill);
        Self::paint_edges(ui, rect);
        Self::paint_center_line(ui, rect, response);
    }

    fn paint_edges(ui: &mut egui::Ui, rect: egui::Rect) {
        let stroke = egui::Stroke::new(
            SPLITTER_STROKE_WIDTH,
            ui.visuals().widgets.hovered.bg_stroke.color,
        );
        let left_x = rect.left() + SPLITTER_EDGE_INSET;
        let right_x = rect.right() - SPLITTER_EDGE_INSET;
        ui.painter().line_segment(
            [
                egui::pos2(left_x, rect.top()),
                egui::pos2(left_x, rect.bottom()),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                egui::pos2(right_x, rect.top()),
                egui::pos2(right_x, rect.bottom()),
            ],
            stroke,
        );
    }

    fn paint_center_line(ui: &mut egui::Ui, rect: egui::Rect, response: &egui::Response) {
        let color = if response.hovered() || response.dragged() {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.noninteractive.fg_stroke.color
        };
        let center_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(SPLITTER_CENTER_WIDTH, rect.height()),
        );
        ui.painter().rect_filled(center_rect, 0.0, color);
    }
}
