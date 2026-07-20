use eframe::egui::{self, Pos2, Vec2};
use katana_document_viewer::browser_session::HtmlBrowserViewport;

pub(super) fn browser_viewport_for_ui(
    size: Vec2,
    pixels_per_point: f32,
) -> Option<HtmlBrowserViewport> {
    let width = physical_dimension(size.x, pixels_per_point)?;
    let height = physical_dimension(size.y, pixels_per_point)?;
    HtmlBrowserViewport::new(width, height, pixels_per_point).ok()
}

pub(super) fn frame_display_size(viewport: HtmlBrowserViewport) -> Vec2 {
    Vec2::new(
        viewport.width as f32 / viewport.device_scale_factor,
        viewport.height as f32 / viewport.device_scale_factor,
    )
}

pub(super) fn frame_position(
    rect: egui::Rect,
    position: Pos2,
    viewport: HtmlBrowserViewport,
) -> Pos2 {
    Pos2::new(
        (position.x - rect.min.x) * viewport.width as f32 / rect.width(),
        (position.y - rect.min.y) * viewport.height as f32 / rect.height(),
    )
}

pub(super) fn frame_scroll_delta(
    rect: egui::Rect,
    delta: Vec2,
    viewport: HtmlBrowserViewport,
) -> Vec2 {
    Vec2::new(
        delta.x * viewport.width as f32 / rect.width(),
        delta.y * viewport.height as f32 / rect.height(),
    )
}

fn physical_dimension(points: f32, pixels_per_point: f32) -> Option<u32> {
    let pixels = (points.max(1.0) * pixels_per_point).round();
    (pixels.is_finite() && (1.0..=u32::MAX as f32).contains(&pixels)).then_some(pixels as u32)
}
