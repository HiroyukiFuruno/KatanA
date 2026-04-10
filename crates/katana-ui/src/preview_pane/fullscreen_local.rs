use crate::icon::Icon;
use crate::preview_pane::ViewerState;
use eframe::egui::{self, Vec2};

use super::fullscreen::{
    FULLSCREEN_CLOSE_MARGIN, FULLSCREEN_CLOSE_SIZE, FULLSCREEN_PADDING, MAX_ZOOM, MIN_ZOOM,
};

pub(super) fn show_fullscreen_local(
    ctx: &egui::Context,
    path: &std::path::Path,
    dc_close: &str,
    viewer_state: &mut ViewerState,
    idx: usize,
) -> bool {
    let screen = ctx.content_rect();
    let mut keep_open = true;

    egui::Area::new(egui::Id::new("fs_input_blocker"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (blocker_rect, response) =
                ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
            if response.hovered() {
                let zoom_delta = ui.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    viewer_state.zoom = (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                }
                viewer_state.pan += if response.dragged() {
                    response.drag_delta()
                } else {
                    ui.input(|i| i.smooth_scroll_delta)
                };
            }
            ui.painter().rect_filled(
                blocker_rect,
                0.0,
                crate::theme_bridge::IMAGE_VIEWER_OVERLAY_COLOR,
            );

            let texture_handle = if viewer_state.texture.is_none() {
                if let Ok(bytes) = std::fs::read(path)
                    && let Ok(dyn_img) = image::load_from_memory(&bytes)
                {
                    let rgba = dyn_img.into_rgba8();
                    let size = std::array::from_fn(|i| {
                        if i == 0 {
                            rgba.width() as usize
                        } else {
                            rgba.height() as usize
                        }
                    });
                    viewer_state.texture = Some(ui.ctx().load_texture(
                        format!("local_image_fs_{idx}"),
                        egui::ColorImage::from_rgba_unmultiplied(size, &rgba),
                        egui::TextureOptions::LINEAR,
                    ));
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
            if render_fs_close_btn(ui, blocker_rect, crate::theme_bridge::WHITE, dc_close) {
                keep_open = false;
            }
        });

    keep_open
}

const CLOSE_BTN_IDLE_OPACITY: f32 = 0.8;

fn render_fs_close_btn(
    ui: &mut egui::Ui,
    blocker_rect: egui::Rect,
    tint: egui::Color32,
    hover_text: &str,
) -> bool {
    let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
    let close_btn_rect = egui::Rect::from_min_size(
        egui::pos2(
            blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
            blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
        ),
        close_btn_size,
    );
    ui.put(
        close_btn_rect,
        egui::Button::image(
            Icon::CloseModal
                .image(crate::icon::IconSize::Large)
                .tint(tint.gamma_multiply(CLOSE_BTN_IDLE_OPACITY)),
        )
        .fill(crate::theme_bridge::TRANSPARENT)
        .stroke(egui::Stroke::NONE),
    )
    .on_hover_text(hover_text)
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .clicked()
}
