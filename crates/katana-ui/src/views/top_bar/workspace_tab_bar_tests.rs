use super::workspace_tab_bar_detail::WorkspaceTabBarDetail;
use super::workspace_tab_bar_detail_layout::WorkspaceTabBarDetailLayout;
use eframe::egui;

#[test]
fn workspace_name_uses_last_path_component() {
    assert_eq!(
        WorkspaceTabBarDetail::workspace_name("/Users/example/workspace/katana"),
        "katana"
    );
}

#[test]
fn tab_width_splits_remaining_width_for_two_tabs() {
    assert_eq!(WorkspaceTabBarDetail::tab_width(1000.0, 2, 40.0), 480.0);
}

#[test]
fn tab_width_splits_remaining_width_for_three_tabs() {
    assert_eq!(WorkspaceTabBarDetail::tab_width(1000.0, 3, 40.0), 320.0);
}

#[test]
fn tab_width_keeps_minimum_width_for_overflow() {
    assert_eq!(WorkspaceTabBarDetail::tab_width(400.0, 4, 40.0), 140.0);
}

#[test]
fn workspace_tab_uses_four_pixel_corner_radius() {
    assert_eq!(
        WorkspaceTabBarDetail::tab_corner_radius(),
        egui::CornerRadius::same(4)
    );
}

#[test]
fn workspace_tab_content_is_centered_inside_parent_tab() {
    let tab_rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(240.0, 28.0));
    let content_rect = WorkspaceTabBarDetailLayout::content_rect(tab_rect, 20.0);

    assert_eq!(content_rect.center().y, tab_rect.center().y);
}

#[test]
fn workspace_title_group_is_centered_as_icon_and_text_unit() {
    let title_rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(240.0, 28.0));
    let group_rect = WorkspaceTabBarDetailLayout::title_group_rect(title_rect, 120.0);

    assert_eq!(group_rect.center().x, title_rect.center().x);
}
