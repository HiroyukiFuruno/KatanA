use super::*;

impl DiagramControllerOps {
    pub(super) fn controls_height() -> f32 {
        BUTTON_SIZE * GRID_DIM + GAP * (GRID_DIM - 1.0) + MARGIN * 2.0
    }

    pub(crate) fn image_and_controls_rects(viewport: egui::Rect) -> (egui::Rect, egui::Rect) {
        let controls_top = (viewport.bottom() - Self::controls_height()).max(viewport.top());
        let image =
            egui::Rect::from_min_max(viewport.min, egui::pos2(viewport.right(), controls_top));
        let controls =
            egui::Rect::from_min_max(egui::pos2(viewport.left(), controls_top), viewport.max);
        (image, controls)
    }

    pub(crate) fn minimum_image_height() -> f32 {
        BUTTON_SIZE + MARGIN * 2.0
    }

    pub(crate) fn draw_controls_below(ui: &mut egui::Ui, state: &mut ViewerState) -> egui::Rect {
        /* WHY: 小さい画像でも全画面ボタンと重ならない独立領域を確保する。 */
        let height = Self::controls_height();
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        Self::draw_controls(ui, state, rect);
        rect
    }

    pub(crate) fn draw_fullscreen_button(ui: &mut egui::Ui, container_rect: egui::Rect) -> bool {
        let msgs = i18n::I18nOps::get();
        let dc = &msgs.preview.diagram_controller;

        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                container_rect.right() - BUTTON_SIZE - MARGIN,
                container_rect.top() + MARGIN,
            ),
            Vec2::splat(BUTTON_SIZE),
        );
        Self::render_control_button(ui, btn_rect, Icon::Fullscreen, &dc.fullscreen, true).clicked()
    }
}
