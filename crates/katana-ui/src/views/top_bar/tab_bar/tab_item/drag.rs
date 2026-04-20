/* WHY: Specialized logic for tab item drag-and-drop orchestration to keep the main item logic clean. */

use crate::views::top_bar::tab_bar::tab_item::TabItem;
use eframe::egui;

impl<'a> TabItem<'a> {
    pub(crate) fn handle_drag(
        &self,
        ui: &mut egui::Ui,
        tab_interact: &egui::Response,
        full_rect: egui::Rect,
        title: &str,
        is_changelog: bool,
    ) -> Option<(egui::Rect, egui::Rangef)> {
        let is_dragged = ui.ctx().is_being_dragged(tab_interact.id);
        let pointer_pos = ui.ctx().pointer_interact_pos()?;
        if !is_dragged {
            return None;
        }
        let press_origin = ui
            .input(|i| i.pointer.press_origin())
            .unwrap_or(pointer_pos);
        let ghost_rect = full_rect.translate(pointer_pos - press_origin);
        ui.memory_mut(|mem| {
            mem.data.insert_temp(
                egui::Id::new("drag_ghost_x").with(self.idx),
                ghost_rect.center().x,
            )
        });
        ui.scroll_to_rect(ghost_rect, None);
        super::super::tab_ghost::render_drag_ghost(
            ui,
            self.idx,
            ghost_rect,
            title,
            is_changelog,
            self.is_active,
            self.doc.is_pinned,
        );
        Some((ghost_rect, full_rect.y_range()))
    }

    pub(crate) fn check_drag_stopped(
        &self,
        ui: &mut egui::Ui,
        tab_interact: &egui::Response,
    ) -> Option<(usize, f32)> {
        if !tab_interact.drag_stopped() {
            return None;
        }
        let ghost_x = ui.memory(|mem| {
            mem.data
                .get_temp::<f32>(egui::Id::new("drag_ghost_x").with(self.idx))
        })?;
        Some((self.idx, ghost_x))
    }
}
