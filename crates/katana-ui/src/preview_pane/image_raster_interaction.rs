use super::{ImageLogicOps, MAX_ZOOM, MIN_ZOOM, ViewerState};
use eframe::egui;

impl ImageLogicOps {
    pub(super) fn apply_raster_pointer_input(
        ui: &egui::Ui,
        state: &mut ViewerState,
        response: &egui::Response,
    ) {
        if response.hovered() {
            let zoom_delta = ui.input(|input| input.zoom_delta());
            if zoom_delta != 1.0 {
                state.zoom = (state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            }
        }
        /* WHY: 押下後に画像領域から出ても、開始済みのドラッグは継続する。 */
        if response.dragged() {
            state.pan += response.drag_delta();
        }
    }
}
