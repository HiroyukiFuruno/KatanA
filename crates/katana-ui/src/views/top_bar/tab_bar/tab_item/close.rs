use crate::views::top_bar::tab_bar::tab_item::TabItem;
use eframe::egui;

impl<'a> TabItem<'a> {
    pub(super) fn close_rect(parent_rect: egui::Rect, close_width: f32) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(
                parent_rect.right() - close_width - Self::close_right_margin(),
                parent_rect.top(),
            ),
            egui::vec2(close_width, parent_rect.height()),
        )
    }

    pub(super) fn is_close_visible(&self, ui: &mut egui::Ui) -> bool {
        ui.memory(|memory| {
            memory
                .data
                .get_temp::<bool>(self.close_visibility_id())
                .unwrap_or(false)
        })
    }

    pub(super) fn set_close_visible(&self, ui: &mut egui::Ui, visible: bool) {
        ui.memory_mut(|memory| {
            memory.data.insert_temp(self.close_visibility_id(), visible);
        });
    }

    fn close_visibility_id(&self) -> egui::Id {
        egui::Id::new("document_tab_close_visible")
            .with(self.idx)
            .with(self.doc.path.to_string_lossy().to_string())
    }
}
