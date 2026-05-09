use super::tab_item::TabItem;
use eframe::egui;

#[test]
fn tab_width_includes_readable_title_padding_and_close_gap() {
    assert_eq!(TabItem::tab_width_from_parts(100.0, 24.0, 0.0, true), 138.0);
}

#[test]
fn tab_width_excludes_close_area_until_hovered() {
    assert_eq!(
        TabItem::tab_width_from_parts(100.0, 24.0, 0.0, false),
        116.0
    );
}

#[test]
fn tab_width_uses_parent_tab_bounds_without_exceeding_max_width() {
    assert_eq!(TabItem::tab_width_from_parts(300.0, 24.0, 0.0, true), 200.0);
}

#[test]
fn tab_width_keeps_minimum_title_area() {
    assert_eq!(TabItem::tab_width_from_parts(0.0, 24.0, 0.0, true), 78.0);
}

#[test]
fn parent_tab_rect_ignores_close_response_height() {
    let parent_rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(110.0, 41.0));
    let taller_close_rect =
        egui::Rect::from_min_max(egui::pos2(90.0, 17.0), egui::pos2(116.0, 45.0));

    assert_eq!(
        TabItem::resolved_parent_tab_rect(parent_rect, Some(taller_close_rect)),
        parent_rect
    );
}
