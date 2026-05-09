use crate::views::top_bar::tab_border::TabBorderOps;
use crate::views::top_bar::tab_drag::TabDragOps;
use crate::views::top_bar::workspace_tab_bar_detail::WorkspaceTabBarDetail;
use eframe::egui;

const WORKSPACE_TAB_DRAG_ID: &str = "workspace_tab_drag_ghost_x";

impl WorkspaceTabBarDetail {
    pub(crate) fn handle_drag(
        ui: &mut egui::Ui,
        index: usize,
        path: &str,
        is_active: bool,
        tab_rect: egui::Rect,
        response: &egui::Response,
    ) -> Option<(egui::Rect, egui::Rangef)> {
        TabDragOps::handle_drag(
            ui,
            WORKSPACE_TAB_DRAG_ID,
            index,
            response,
            tab_rect,
            |ui, ghost_rect| Self::render_drag_ghost(ui, index, path, is_active, ghost_rect),
        )
    }

    pub(crate) fn check_drag_stopped(
        ui: &mut egui::Ui,
        index: usize,
        response: &egui::Response,
    ) -> Option<(usize, f32)> {
        TabDragOps::check_drag_stopped(ui, WORKSPACE_TAB_DRAG_ID, index, response)
    }

    fn render_drag_ghost(
        ui: &mut egui::Ui,
        index: usize,
        path: &str,
        is_active: bool,
        ghost_rect: egui::Rect,
    ) {
        egui::Area::new(egui::Id::new("workspace_tab_ghost").with(index))
            .fixed_pos(ghost_rect.min)
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                let local_rect = egui::Rect::from_min_size(ui.cursor().min, ghost_rect.size());
                let title = Self::workspace_name(path);
                Self::render_title(ui, local_rect, title, is_active).on_hover_text(path);
                TabBorderOps::paint_with_radius(ui, local_rect, true, Self::tab_corner_radius());
            });
    }
}
