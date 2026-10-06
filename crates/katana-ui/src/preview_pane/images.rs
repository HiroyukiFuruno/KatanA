use super::image_raster::{MAX_ZOOM, MIN_ZOOM};
use crate::preview_pane::{ViewerState, ViewerTextureIdentity};
use eframe::egui::{self, Vec2};

const IMAGE_LOADING_REPAINT_INTERVAL_MS: u64 = 16;

pub use super::types::ImageLogicOps;

impl ImageLogicOps {
    pub(crate) fn show_local_image(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        id: usize,
        loader: &super::local_image_loader::LocalImageLoader,
        mut viewer_state: Option<&mut ViewerState>,
        fullscreen_request: Option<&mut Option<usize>>,
        draw_background: impl FnOnce(&mut egui::Ui, egui::Rect, bool),
    ) -> Option<egui::Rect> {
        let preview_background = super::image_background::ImageBackgroundOps::preview_background(
            ui.ctx(),
            ui.visuals().window_fill(),
        );
        loader.set_repaint_context(ui.ctx());
        let texture_handle = if let Some(state) = viewer_state.as_mut() {
            state.prepare_texture(
                ViewerTextureIdentity::local_file_revision(path, loader.path_revision(path)),
                preview_background,
            );
            if state.texture.is_none() || state.texture_background != Some(preview_background) {
                match loader.request(path, preview_background) {
                    super::local_image_loader::LocalImageStatus::Ready(image) => {
                        state.texture = Some(ui.ctx().load_texture(
                            format!("local_image_{id}"),
                            image,
                            egui::TextureOptions::LINEAR,
                        ));
                        state.texture_background = Some(preview_background);
                        loader.release_active_image(path, preview_background);
                    }
                    super::local_image_loader::LocalImageStatus::Pending => {
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_millis(
                                IMAGE_LOADING_REPAINT_INTERVAL_MS,
                            ));
                        ui.label(&crate::i18n::I18nOps::get().preview.rendering);
                    }
                    super::local_image_loader::LocalImageStatus::Failed(error) => {
                        ui.label(&crate::i18n::I18nOps::get().preview.missing_image)
                            .on_hover_text(error);
                    }
                }
            }
            state.texture.clone()
        } else {
            match loader.texture(ui.ctx(), path, preview_background, id) {
                super::local_image_loader::LocalTextureStatus::Ready(texture) => Some(texture),
                super::local_image_loader::LocalTextureStatus::Pending => {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(
                            IMAGE_LOADING_REPAINT_INTERVAL_MS,
                        ));
                    ui.label(&crate::i18n::I18nOps::get().preview.rendering);
                    None
                }
                super::local_image_loader::LocalTextureStatus::Failed(error) => {
                    ui.label(&crate::i18n::I18nOps::get().preview.missing_image)
                        .on_hover_text(error);
                    None
                }
            }
        };

        let (texture_handle, width, height) = match texture_handle {
            Some(t) => {
                let size = t.size();
                (t, size[0], size[1])
            }
            None => {
                return None;
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
        super::image_background::ImageBackgroundOps::paint(ui, container_rect);

        response.context_menu(|ui| {
            if ui
                .button(&crate::i18n::I18nOps::get().action.reveal_in_os)
                .clicked()
            {
                let _ = open::that(path);
                ui.close();
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
            #[cfg(feature = "screenshot-test-hooks")]
            crate::preview_pane::overlay_inspection::PreviewOverlayInspectionOps::increment(
                ui.ctx(),
                |inspection| {
                    inspection.image_control_renders += 1;
                },
            );
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
