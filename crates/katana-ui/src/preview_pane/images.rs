use super::image_raster::{MAX_ZOOM, MIN_ZOOM};
use crate::preview_pane::ViewerState;
use eframe::egui::{self, TextureHandle, Vec2};

pub use super::types::ImageLogicOps;

fn load_local_image_texture(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    id: usize,
) -> Option<TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    let color_img = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
    {
        let svg = std::str::from_utf8(&bytes).ok()?;
        let rasterized =
            katana_core::markdown::svg_rasterize::SvgRasterizeOps::rasterize_svg(svg, 1.0).ok()?;
        egui::ColorImage::from_rgba_unmultiplied(
            std::array::from_fn(|i| {
                if i == 0 {
                    rasterized.width as usize
                } else {
                    rasterized.height as usize
                }
            }),
            &rasterized.rgba,
        )
    } else {
        let rgba = image::load_from_memory(&bytes).ok()?.into_rgba8();
        let size = std::array::from_fn(|i| {
            if i == 0 {
                rgba.width() as usize
            } else {
                rgba.height() as usize
            }
        });
        egui::ColorImage::from_rgba_unmultiplied(size, &rgba)
    };

    Some(ui.ctx().load_texture(
        format!("local_image_{id}"),
        color_img,
        egui::TextureOptions::LINEAR,
    ))
}

impl ImageLogicOps {
    pub(crate) fn show_local_image(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        _alt: &str,
        id: usize,
        mut viewer_state: Option<&mut ViewerState>,
        fullscreen_request: Option<&mut Option<usize>>,
        draw_background: impl FnOnce(&mut egui::Ui, egui::Rect, bool),
    ) -> Option<egui::Rect> {
        let texture_handle = if let Some(state) = viewer_state.as_mut() {
            if state.texture.is_none() {
                state.texture = load_local_image_texture(ui, path, id);
            }
            state.texture.clone()
        } else {
            None
        };

        let (texture_handle, width, height) = match texture_handle {
            Some(t) => {
                let size = t.size();
                (t, size[0], size[1])
            }
            None => {
                return super::image_fallback::ImageFallbackOps::show_image_fallback(ui, path);
            }
        };

        let max_w = ui.available_width();
        let base_scale = (max_w / width as f32).min(1.0);

        let zoom = viewer_state.as_ref().map_or(1.0, |s| s.zoom);
        let pan = viewer_state.as_ref().map_or(egui::Vec2::ZERO, |s| s.pan);

        let base_size = Vec2::new(width as f32 * base_scale, height as f32 * base_scale);
        let zoomed_size = base_size * zoom;

        let (container_rect, response) =
            ui.allocate_exact_size(Vec2::new(max_w, base_size.y), egui::Sense::click_and_drag());

        response.context_menu(|ui| {
            if ui
                .button(&crate::i18n::I18nOps::get().action.reveal_in_os)
                .clicked()
            {
                let _ = open::that(path);
                ui.close_menu();
            }
        });

        draw_background(ui, container_rect, response.hovered());

        if let Some(state) = viewer_state.as_mut()
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

        let x_offset = (max_w - base_size.x).max(0.0) / 2.0;
        let image_pos = container_rect.min + egui::vec2(x_offset, 0.0) + pan;
        let image_rect = egui::Rect::from_min_size(image_pos, zoomed_size);

        ui.painter().with_clip_rect(container_rect).image(
            texture_handle.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            crate::theme_bridge::WHITE,
        );

        if let Some(state) = viewer_state {
            if crate::diagram_controller::DiagramControllerOps::draw_fullscreen_button(
                ui,
                container_rect,
            ) && let Some(req) = fullscreen_request
            {
                *req = Some(id);
            }
            crate::diagram_controller::DiagramControllerOps::draw_controls(
                ui,
                state,
                container_rect,
            );
        }

        Some(container_rect)
    }
}
