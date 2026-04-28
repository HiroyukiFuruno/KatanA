use super::split::SplitLayout;
use eframe::egui;

const SPLIT_MIN_RATIO: f32 = 0.1_f32;
const SPLIT_MAX_RATIO: f32 = 0.9_f32;
const SPLITTER_WIDTH: f32 = 6.0_f32;

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
        let color = if response.hovered() || response.dragged() {
            ui.visuals().widgets.active.bg_stroke.color
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke.color
        };
        ui.painter().rect_filled(rect, 0.0, color);
    }
}
