use crate::app_state::AppAction;
use eframe::egui;

const POPUP_ID: &str = "editor_authoring_toolbar_popup";
const POPUP_GAP: f32 = 6.0;
const POPUP_EDGE_PADDING: f32 = 8.0;
const POPUP_ESTIMATED_WIDTH: f32 = 352.0;
const POPUP_ESTIMATED_HEIGHT: f32 = 44.0;
const POPUP_MARGIN: i8 = 4;

pub(crate) struct ToolbarPopup;

impl ToolbarPopup {
    pub(crate) fn show(
        ui: &mut egui::Ui,
        action: &mut AppAction,
        response: &egui::Response,
        galley: &egui::text::Galley,
        cursor_range: Option<egui::text::CCursorRange>,
        editable: bool,
    ) {
        if !editable || !Self::editor_has_input_focus(ui, response) {
            return;
        }

        let Some(cursor_range) = cursor_range else {
            return;
        };

        let cursor_rect = galley.pos_from_cursor(cursor_range.primary);
        let popup_pos = Self::popup_position(response.rect, cursor_rect, ui.ctx().content_rect());
        let has_selection = cursor_range.primary.index != cursor_range.secondary.index;

        egui::Area::new(egui::Id::new(POPUP_ID))
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::same(POPUP_MARGIN))
                    .show(ui, |ui| {
                        super::toolbar::EditorToolbar::new(action, has_selection).show(ui);
                    });
            });
    }

    fn editor_has_input_focus(ui: &egui::Ui, response: &egui::Response) -> bool {
        response.has_focus() || ui.memory(|mem| mem.focused().is_some_and(|id| id == response.id))
    }

    fn popup_position(
        response_rect: egui::Rect,
        cursor_rect: egui::Rect,
        viewport_rect: egui::Rect,
    ) -> egui::Pos2 {
        let desired_x = response_rect.min.x + cursor_rect.min.x;
        let desired_y = response_rect.min.y + cursor_rect.max.y + POPUP_GAP;
        let min_x = viewport_rect.min.x + POPUP_EDGE_PADDING;
        let min_y = viewport_rect.min.y + POPUP_EDGE_PADDING;
        let max_x = (viewport_rect.max.x - POPUP_ESTIMATED_WIDTH - POPUP_EDGE_PADDING).max(min_x);
        let max_y = (viewport_rect.max.y - POPUP_ESTIMATED_HEIGHT - POPUP_EDGE_PADDING).max(min_y);

        egui::pos2(desired_x.clamp(min_x, max_x), desired_y.clamp(min_y, max_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_position_anchors_below_cursor() {
        let response = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(600.0, 400.0));
        let cursor = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(2.0, 18.0));
        let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));

        let actual = ToolbarPopup::popup_position(response, cursor, viewport);

        assert_eq!(actual, egui::pos2(110.0, 94.0));
    }

    #[test]
    fn popup_position_stays_inside_viewport_edges() {
        let response = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(600.0, 400.0));
        let cursor = egui::Rect::from_min_size(egui::pos2(760.0, 570.0), egui::vec2(2.0, 18.0));
        let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));

        let actual = ToolbarPopup::popup_position(response, cursor, viewport);

        assert_eq!(actual, egui::pos2(440.0, 548.0));
    }
}
