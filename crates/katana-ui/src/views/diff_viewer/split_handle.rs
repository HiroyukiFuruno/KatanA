use super::split::SplitLayout;
use eframe::egui;

const SPLIT_MIN_RATIO: f32 = 0.1_f32;
const SPLIT_MAX_RATIO: f32 = 0.9_f32;
const SPLITTER_WIDTH: f32 = 8.0_f32;
const SPLITTER_LINE_WIDTH: f32 = 1.0_f32;

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
        if response.hovered() || response.dragged() {
            ui.painter()
                .rect_filled(rect, 0.0, ui.visuals().widgets.hovered.bg_fill);
        }
        Self::paint_center_line(ui, rect, response);
    }

    fn paint_center_line(ui: &mut egui::Ui, rect: egui::Rect, response: &egui::Response) {
        let stroke = if response.hovered() || response.dragged() {
            egui::Stroke::new(
                SPLITTER_LINE_WIDTH,
                ui.visuals().widgets.hovered.bg_stroke.color,
            )
        } else {
            ui.visuals().window_stroke()
        };
        ui.painter().line_segment(
            [
                egui::pos2(rect.center().x, rect.top()),
                egui::pos2(rect.center().x, rect.bottom()),
            ],
            stroke,
        );
    }
}
