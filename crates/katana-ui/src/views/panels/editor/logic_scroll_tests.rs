#[cfg(test)]
mod tests {
    use crate::views::panels::editor::types::EditorLogicOps;

    #[test]
    fn jump_target_scrolls_line_to_center() {
        let response_rect =
            egui::Rect::from_min_max(egui::pos2(10.0, 100.0), egui::pos2(310.0, 500.0));
        let cursor_rect = egui::Rect::from_min_max(egui::pos2(0.0, 40.0), egui::pos2(120.0, 56.0));

        let (target_rect, alignment) =
            EditorLogicOps::jump_target_scroll_rect(&response_rect, cursor_rect);

        assert_eq!(alignment, egui::Align::Center);
        assert_eq!(
            target_rect,
            egui::Rect::from_min_max(egui::pos2(10.0, 140.0), egui::pos2(310.0, 156.0))
        );
    }
}
