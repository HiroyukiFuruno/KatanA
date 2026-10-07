use crate::preview_pane::{ViewerState, ViewerTextureIdentity};
use eframe::egui::{self, Vec2};

use super::fullscreen::FULLSCREEN_PADDING;

const REPAINT_INTERVAL_MS: u64 = 16;

pub(super) fn show_fullscreen_local(
    ctx: &egui::Context,
    path: &std::path::Path,
    dc_close: &str,
    viewer_state: &mut ViewerState,
    idx: usize,
    loader: &super::local_image_loader::LocalImageLoader,
) -> bool {
    loader.set_repaint_context(ctx);
    let screen = ctx.content_rect();
    let mut keep_open = true;

    egui::Area::new(egui::Id::new("fs_input_blocker"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let panel_fill = ui.visuals().panel_fill;
            let background = crate::theme_bridge::ThemeBridgeOps::from_rgb(
                panel_fill.r(),
                panel_fill.g(),
                panel_fill.b(),
            );
            viewer_state.prepare_texture(
                ViewerTextureIdentity::local_file_revision(path, loader.path_revision(path)),
                background,
            );
            let (blocker_rect, response) =
                ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
            if response.hovered() {
                super::fullscreen::FullscreenInteraction::from_input(ui, &response)
                    .apply(viewer_state);
            }
            ui.painter().rect_filled(blocker_rect, 0.0, background);

            let texture_handle = if viewer_state.texture.is_none() {
                match loader.request(path, background) {
                    super::local_image_loader::LocalImageStatus::Ready(image) => {
                        viewer_state.texture = Some(ui.ctx().load_texture(
                            format!("local_image_{idx}"),
                            image,
                            egui::TextureOptions::LINEAR,
                        ));
                        viewer_state.texture_background = Some(background);
                        loader.release_active_image(path, background);
                    }
                    super::local_image_loader::LocalImageStatus::Pending => {
                        ctx.request_repaint_after(std::time::Duration::from_millis(
                            REPAINT_INTERVAL_MS,
                        ));
                        ui.label(&crate::i18n::I18nOps::get().preview.rendering);
                    }
                    super::local_image_loader::LocalImageStatus::Failed(error) => {
                        ui.label(&crate::i18n::I18nOps::get().preview.missing_image)
                            .on_hover_text(error);
                    }
                }
                viewer_state.texture.clone()
            } else {
                viewer_state.texture.clone()
            };

            let (texture_handle, width, height) = match texture_handle {
                Some(t) => {
                    let s = t.size();
                    (t, s[0], s[1])
                }
                None => return,
            };

            let avail = Vec2::new(
                screen.width() - FULLSCREEN_PADDING * 2.0,
                screen.height() - FULLSCREEN_PADDING * 2.0,
            );
            let base_scale = (avail.x / width as f32)
                .min(avail.y / height as f32)
                .min(1.0);
            let size = Vec2::new(
                width as f32 * base_scale * viewer_state.zoom,
                height as f32 * base_scale * viewer_state.zoom,
            );
            let img_pos = screen.center() - size / 2.0 + viewer_state.pan;
            ui.painter().with_clip_rect(blocker_rect).image(
                texture_handle.id(),
                egui::Rect::from_min_size(img_pos, size),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                crate::theme_bridge::WHITE,
            );
            crate::diagram_controller::DiagramControllerOps::draw_controls(
                ui,
                viewer_state,
                blocker_rect,
            );
            if super::fullscreen::render_fullscreen_close_button(ui, blocker_rect, dc_close, 1.0) {
                keep_open = false;
            }
        });

    keep_open
}
