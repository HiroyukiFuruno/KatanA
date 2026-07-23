use super::HtmlBrowserSurface;
use eframe::egui::{self, Pos2, Vec2};
use katana_document_viewer::browser_session::HtmlBrowserViewport;

impl HtmlBrowserSurface {
    pub(super) fn resize_to_ui(&mut self, ui: &egui::Ui) {
        let Some(adapter) = &self.adapter else {
            return;
        };
        let size = ui.available_size().max(Vec2::splat(1.0));
        let max_texture_side = ui.ctx().input(|input| input.max_texture_side);
        let Some(viewport) =
            browser_viewport_for_ui(size, ui.ctx().pixels_per_point(), max_texture_side)
        else {
            return;
        };
        if self.viewport == Some(viewport) {
            return;
        }
        if let Err(error) = adapter.resize(viewport) {
            self.record_adapter_error("resize", None, error);
            return;
        }
        self.discard_bootstrap_frame(viewport);
        self.viewport = Some(viewport);
        self.await_frame();
    }
}

pub(super) fn browser_viewport_for_ui(
    size: Vec2,
    pixels_per_point: f32,
    max_texture_side: usize,
) -> Option<HtmlBrowserViewport> {
    let logical_size = size.max(Vec2::splat(1.0));
    let requested_max_side = logical_size.max_elem() * pixels_per_point;
    let texture_scale = (max_texture_side as f32 / requested_max_side).min(1.0);
    let effective_scale = pixels_per_point * texture_scale;
    let width = physical_dimension(logical_size.x, effective_scale)?;
    let height = physical_dimension(logical_size.y, effective_scale)?;
    HtmlBrowserViewport::new(width, height, effective_scale).ok()
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
