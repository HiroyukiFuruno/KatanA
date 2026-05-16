use crate::preview_pane::{ViewerState, ViewerTextureIdentity};
use eframe::egui::{self, Vec2};
use image::{ImageBuffer, Rgba};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use super::types::ImageLogicOps;

pub(super) const MIN_ZOOM: f32 = 0.1;
pub(super) const MAX_ZOOM: f32 = 10.0;

/// WHY: Minimum container height to prevent the 3×3 control grid and fullscreen
/// button from overlapping on diagrams with a small rendered height.
const MIN_CONTAINER_HEIGHT: f32 = 145.0;
const MAX_TEXTURE_SIDE: usize = 2048;

fn color_image_for_texture(
    img: &RasterizedSvg,
    background: egui::Color32,
    should_replace_light_background: bool,
) -> egui::ColorImage {
    let width = img.width as usize;
    let height = img.height as usize;
    let max_side = width.max(height);
    let mut rgba = img.rgba.clone();
    super::image_background::ImageBackgroundOps::composite_rgba_over_background(
        &mut rgba, background,
    );
    if should_replace_light_background {
        super::image_background_region::ImageBackgroundRegionOps::replace_large_light_regions(
            &mut rgba, width, height, background,
        );
    }

    if max_side <= MAX_TEXTURE_SIDE {
        return egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
    }

    let scale = MAX_TEXTURE_SIDE as f32 / max_side as f32;
    let resized_width = (width as f32 * scale).max(1.0).round() as u32;
    let resized_height = (height as f32 * scale).max(1.0).round() as u32;

    let Some(src) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, rgba)
    else {
        return egui::ColorImage::from_rgba_unmultiplied([width, height], &img.rgba);
    };

    let resized = image::imageops::resize(
        &src,
        resized_width,
        resized_height,
        image::imageops::FilterType::Triangle,
    );
    let pixels = resized.into_raw();
    egui::ColorImage::from_rgba_unmultiplied(
        [resized_width as usize, resized_height as usize],
        &pixels,
    )
}

impl ImageLogicOps {
    pub(crate) fn show_rasterized(
        ui: &mut egui::Ui,
        img: &RasterizedSvg,
        alt_text: &str,
        idx: usize,
        mut state: Option<&mut ViewerState>,
        fullscreen_request: Option<&mut Option<usize>>,
        draw_background: impl FnOnce(&mut egui::Ui, egui::Rect, bool),
    ) -> egui::Rect {
        let max_w = ui.available_width();
        let display_width = img.display_width.max(1.0);
        let display_height = img.display_height.max(1.0);
        let base_scale = (max_w / display_width).min(1.0);
        let base_size = Vec2::new(display_width * base_scale, display_height * base_scale);
        let container_h = base_size.y.max(MIN_CONTAINER_HEIGHT);
        let (container_rect, response) =
            ui.allocate_exact_size(Vec2::new(max_w, container_h), egui::Sense::click_and_drag());
        super::image_background::ImageBackgroundOps::paint(ui, container_rect);
        let preview_background = super::image_background::ImageBackgroundOps::preview_background(
            ui.ctx(),
            ui.visuals().window_fill(),
        );

        if let Some(state) = state.as_mut() {
            state.prepare_texture(ViewerTextureIdentity::rasterized(img), preview_background);
        }

        if let Some(state) = state.as_mut()
            && response.hovered()
        {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                state.zoom = (state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            }
            if response.dragged() {
                state.pan += response.drag_delta();
            }
        }

        let zoom = state.as_ref().map_or(1.0, |s| s.zoom);
        let pan = state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);
        let zoomed_size = base_size * zoom;

        let texture_handle = if let Some(state) = state.as_mut() {
            if state.texture.is_none() || state.texture_background != Some(preview_background) {
                let color_img = color_image_for_texture(
                    img,
                    preview_background,
                    should_replace_light_background(alt_text),
                );
                state.texture = Some(ui.ctx().load_texture(
                    format!("diagram_{idx}"),
                    color_img,
                    egui::TextureOptions::LINEAR,
                ));
                state.texture_background = Some(preview_background);
            }
            state.texture.clone().unwrap()
        } else {
            let color_img = color_image_for_texture(
                img,
                preview_background,
                should_replace_light_background(alt_text),
            );
            ui.ctx().load_texture(
                format!("diagram_{idx}"),
                color_img,
                egui::TextureOptions::LINEAR,
            )
        };

        let x_offset = (max_w - base_size.x).max(0.0) / 2.0;
        let y_offset = (container_h - base_size.y).max(0.0) / 2.0;
        let image_pos = container_rect.min + egui::vec2(x_offset, y_offset) + pan;
        let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);
        ui.painter().with_clip_rect(container_rect).image(
            texture_handle.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            crate::theme_bridge::WHITE,
        );

        draw_background(ui, container_rect, response.hovered());

        if let Some(state) = state {
            if crate::diagram_controller::DiagramControllerOps::draw_fullscreen_button(
                ui,
                container_rect,
            ) && let Some(req) = fullscreen_request
            {
                *req = Some(idx);
            }
            crate::diagram_controller::DiagramControllerOps::draw_controls(
                ui,
                state,
                container_rect,
            );
        }

        container_rect
    }
}

fn should_replace_light_background(alt_text: &str) -> bool {
    alt_text == "ZenUML diagram"
}
