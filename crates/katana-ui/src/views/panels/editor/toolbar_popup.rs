use crate::app_state::AppAction;
use eframe::egui;

const POPUP_ID: &str = "editor_authoring_toolbar_popup";
const POPUP_OPEN_ID: &str = "editor_authoring_toolbar_popup_open";
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
        suppress: bool,
    ) {
        let editor_focused = Self::editor_has_input_focus(ui, response);
        let was_open = ui.memory(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new(POPUP_OPEN_ID))
                .unwrap_or(false)
        });
        if !Self::should_show(
            editable,
            editor_focused,
            was_open,
            cursor_range.is_some(),
            suppress,
        ) {
            Self::store_open(ui, false);
            return;
        }

        let Some(cursor_range) = cursor_range else {
            return;
        };

        let cursor_rect = galley.pos_from_cursor(cursor_range.primary);
        let popup_pos = Self::popup_position(response.rect, cursor_rect, ui.ctx().content_rect());
        let has_selection = cursor_range.primary.index != cursor_range.secondary.index;

        let area_response = egui::Area::new(egui::Id::new(POPUP_ID))
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style())
                    .inner_margin(egui::Margin::same(POPUP_MARGIN))
                    .show(ui, |ui| {
                        super::toolbar::EditorToolbar::new(action, has_selection).show(ui);
                    })
                    .response
                    .hovered()
            });
        Self::store_open(
            ui,
            editor_focused
                || area_response.inner
                || super::code_block_menu::CodeBlockMenuPopupOps::is_open(ui),
        );
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

    fn should_show(
        editable: bool,
        editor_focused: bool,
        was_open: bool,
        has_cursor: bool,
        suppress: bool,
    ) -> bool {
        editable && has_cursor && !suppress && (editor_focused || was_open)
    }

    fn store_open(ui: &egui::Ui, open: bool) {
        ui.memory_mut(|mem| mem.data.insert_temp(egui::Id::new(POPUP_OPEN_ID), open));
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

    #[test]
    fn popup_stays_available_after_editor_focus_moves_to_toolbar() {
        assert!(ToolbarPopup::should_show(true, false, true, true, false));
    }

    #[test]
    fn popup_does_not_show_without_cursor() {
        assert!(!ToolbarPopup::should_show(true, true, false, false, false));
    }

    #[test]
    fn popup_does_not_show_while_diagnostic_hover_is_active() {
        assert!(!ToolbarPopup::should_show(true, true, true, true, true));
    }

    #[test]
    fn popup_stays_available_when_child_menu_is_open() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::super::code_block_menu::CodeBlockMenuPopupOps::set_open(ui, true);

                ToolbarPopup::store_open(
                    ui,
                    super::super::code_block_menu::CodeBlockMenuPopupOps::is_open(ui),
                );

                let was_open = ui.memory(|mem| {
                    mem.data
                        .get_temp::<bool>(egui::Id::new(POPUP_OPEN_ID))
                        .unwrap_or(false)
                });
                assert!(was_open);
            });
        });
    }
}
