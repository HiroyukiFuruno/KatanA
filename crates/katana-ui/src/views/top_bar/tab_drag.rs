use eframe::egui;

pub(crate) struct TabDragOps;

impl TabDragOps {
    pub(crate) fn handle_drag(
        ui: &mut egui::Ui,
        id_salt: &'static str,
        index: usize,
        response: &egui::Response,
        tab_rect: egui::Rect,
        render_ghost: impl FnOnce(&mut egui::Ui, egui::Rect),
    ) -> Option<(egui::Rect, egui::Rangef)> {
        let is_dragged = ui.ctx().is_being_dragged(response.id);
        let pointer_pos = ui.ctx().pointer_interact_pos()?;
        if !is_dragged {
            return None;
        }
        let press_origin = ui
            .input(|input| input.pointer.press_origin())
            .unwrap_or(pointer_pos);
        let ghost_rect = tab_rect.translate(pointer_pos - press_origin);
        ui.memory_mut(|memory| {
            memory
                .data
                .insert_temp(Self::ghost_x_id(id_salt, index), ghost_rect.center().x);
        });
        ui.scroll_to_rect(ghost_rect, None);
        render_ghost(ui, ghost_rect);
        Some((ghost_rect, tab_rect.y_range()))
    }

    pub(crate) fn check_drag_stopped(
        ui: &mut egui::Ui,
        id_salt: &'static str,
        index: usize,
        response: &egui::Response,
    ) -> Option<(usize, f32)> {
        if !response.drag_stopped() {
            return None;
        }
        let ghost_x = ui.memory(|memory| {
            memory
                .data
                .get_temp::<f32>(Self::ghost_x_id(id_salt, index))
        })?;
        Some((index, ghost_x))
    }

    fn ghost_x_id(id_salt: &'static str, index: usize) -> egui::Id {
        egui::Id::new(id_salt).with(index)
    }
}
