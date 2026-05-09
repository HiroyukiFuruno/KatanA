use eframe::egui;

const TITLE_PADDING_X: f32 = 8.0;
const ICON_GAP: f32 = 4.0;
const ICON_BASELINE_OFFSET_Y: f32 = 1.0;

pub(crate) struct WorkspaceTabBarDetailLayout;

impl WorkspaceTabBarDetailLayout {
    pub(crate) fn content_rect(tab_rect: egui::Rect, content_height: f32) -> egui::Rect {
        let height = content_height.min(tab_rect.height());
        let size = egui::vec2(tab_rect.width(), height);
        egui::Rect::from_center_size(tab_rect.center(), size)
    }

    pub(crate) fn title_group_rect(title_rect: egui::Rect, group_width: f32) -> egui::Rect {
        let width = group_width.min(title_rect.width()).max(0.0);
        egui::Rect::from_center_size(title_rect.center(), egui::vec2(width, title_rect.height()))
    }

    pub(crate) fn max_group_width(title_width: f32) -> f32 {
        (title_width - TITLE_PADDING_X * 2.0).max(0.0)
    }

    pub(crate) fn icon_gap() -> f32 {
        ICON_GAP
    }

    pub(crate) fn icon_baseline_offset_y() -> f32 {
        ICON_BASELINE_OFFSET_Y
    }
}
