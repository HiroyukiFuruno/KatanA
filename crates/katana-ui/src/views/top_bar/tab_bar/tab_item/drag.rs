/* WHY: Specialized logic for tab item drag-and-drop orchestration to keep the main item logic clean. */

use crate::views::top_bar::tab_bar::tab_item::TabItem;
use crate::views::top_bar::tab_drag::TabDragOps;
use eframe::egui;

const DOCUMENT_TAB_DRAG_ID: &str = "document_tab_drag_ghost_x";

impl<'a> TabItem<'a> {
    pub(crate) fn handle_drag(
        &self,
        ui: &mut egui::Ui,
        tab_interact: &egui::Response,
        full_rect: egui::Rect,
        title: &str,
        is_changelog: bool,
    ) -> Option<(egui::Rect, egui::Rangef)> {
        TabDragOps::handle_drag(
            ui,
            DOCUMENT_TAB_DRAG_ID,
            self.idx,
            tab_interact,
            full_rect,
            |ui, ghost_rect| {
                super::super::tab_ghost::render_drag_ghost(
                    ui,
                    self.idx,
                    ghost_rect,
                    title,
                    is_changelog,
                    self.is_active,
                    self.doc.is_pinned,
                );
            },
        )
    }

    pub(crate) fn check_drag_stopped(
        &self,
        ui: &mut egui::Ui,
        tab_interact: &egui::Response,
    ) -> Option<(usize, f32)> {
        TabDragOps::check_drag_stopped(ui, DOCUMENT_TAB_DRAG_ID, self.idx, tab_interact)
    }
}
