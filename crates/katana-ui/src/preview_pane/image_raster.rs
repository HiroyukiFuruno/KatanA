use crate::preview_pane::ViewerState;
use eframe::egui::{self, Vec2};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use super::types::ImageLogicOps;

pub(super) const MIN_ZOOM: f32 = 0.1;
pub(super) const MAX_ZOOM: f32 = 10.0;

impl ImageLogicOps {
    pub(crate) fn show_rasterized(
        ui: &mut egui::Ui,
        img: &RasterizedSvg,
        _alt_text: &str,
        idx: usize,
        mut state: Option<&mut ViewerState>,
        fullscreen_request: Option<&mut Option<usize>>,
    ) -> egui::Rect {
        let max_w = ui.available_width();
        let base_scale = (max_w / img.width as f32).min(1.0);
        let zoom = state.as_ref().map_or(1.0, |s| s.zoom);
        let pan = state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);
        let base_size = Vec2::new(
            img.width as f32 * base_scale,
            img.height as f32 * base_scale,
        );
        let zoomed_size = base_size * zoom;
        let (container_rect, response) =
            ui.allocate_exact_size(Vec2::new(max_w, base_size.y), egui::Sense::click_and_drag());

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

        let texture_handle = if let Some(state) = state.as_mut() {
            if state.texture.is_none() {
                let color_img = egui::ColorImage::from_rgba_unmultiplied(
                    std::array::from_fn(|i| {
                        if i == 0 {
                            img.width as usize
                        } else {
                            img.height as usize
                        }
                    }),
                    &img.rgba,
                );
                state.texture = Some(ui.ctx().load_texture(
                    format!("diagram_{idx}"),
                    color_img,
                    egui::TextureOptions::LINEAR,
                ));
            }
            state.texture.clone().unwrap()
        } else {
            let color_img = egui::ColorImage::from_rgba_unmultiplied(
                std::array::from_fn(|i| {
                    if i == 0 {
                        img.width as usize
                    } else {
                        img.height as usize
                    }
                }),
                &img.rgba,
            );
            ui.ctx().load_texture(
                format!("diagram_{idx}"),
                color_img,
                egui::TextureOptions::LINEAR,
            )
        };

        let x_offset = (max_w - base_size.x).max(0.0) / 2.0;
        let image_pos = container_rect.min + egui::vec2(x_offset, 0.0) + pan;
        let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);
        ui.painter().with_clip_rect(container_rect).image(
            texture_handle.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            crate::theme_bridge::WHITE,
        );

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
