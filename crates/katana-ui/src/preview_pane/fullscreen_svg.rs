use crate::icon::Icon;
use crate::preview_pane::ViewerState;
use eframe::egui::{self, Vec2};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use super::fullscreen::{
    CLOSE_BTN_IDLE_OPACITY, FULLSCREEN_CLOSE_MARGIN, FULLSCREEN_CLOSE_SIZE, FULLSCREEN_PADDING,
    MAX_ZOOM, MIN_ZOOM,
};

pub(super) fn show_fullscreen_svg(
    ctx: &egui::Context,
    img: &RasterizedSvg,
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
            let current_time = ui.input(|i| i.time);
            let mut alpha = 1.0;
            if let Some(start_time) = viewer_state.closing_since {
                let progress = ((current_time - start_time) as f32)
                    / super::fullscreen::FULLSCREEN_FADE_OUT_DURATION;
                if progress >= 1.0 {
                    keep_open = false;
                } else {
                    alpha = 1.0 - progress;
                    ctx.request_repaint();
                }
            }

            let (blocker_rect, response) =
                ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
            if response.hovered() && viewer_state.closing_since.is_none() {
                let zoom_delta = ui.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    viewer_state.zoom = (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                }
                viewer_state.pan += if response.dragged() {
                    response.drag_delta()
                } else {
                    ui.input(|i| i.smooth_scroll_delta)
                };
                if response.clicked() {
                    viewer_state.closing_since = Some(current_time);
                }
            }

            let c = ui.visuals().panel_fill;
            let mut bg_color = crate::theme_bridge::ThemeBridgeOps::from_rgb(c.r(), c.g(), c.b());
            /* WHY: Support fade out animation by multiplying alpha. */
            bg_color = bg_color.gamma_multiply(alpha);

            ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

            let avail = Vec2::new(
                screen.width() - FULLSCREEN_PADDING * 2.0,
                screen.height() - FULLSCREEN_PADDING * 2.0,
            );
            let base_scale = (avail.x / img.width as f32)
                .min(avail.y / img.height as f32)
                .min(1.0);
            let size = Vec2::new(
                img.width as f32 * base_scale * viewer_state.zoom,
                img.height as f32 * base_scale * viewer_state.zoom,
            );

            let texture_handle = viewer_state
                .texture
                .get_or_insert_with(|| {
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
                    ctx.load_texture(
                        format!("diagram_fs_{idx}"),
                        color_img,
                        egui::TextureOptions::LINEAR,
                    )
                })
                .clone();

            let img_pos = screen.center() - size / 2.0 + viewer_state.pan;
            ui.painter().with_clip_rect(blocker_rect).image(
                texture_handle.id(),
                egui::Rect::from_min_size(img_pos, size),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                crate::theme_bridge::WHITE.gamma_multiply(alpha),
            );
            crate::diagram_controller::DiagramControllerOps::draw_controls(
                ui,
                viewer_state,
                blocker_rect,
            );
            if render_fs_close_btn(
                ui,
                blocker_rect,
                crate::theme_bridge::WHITE.gamma_multiply(alpha),
                dc_close,
            ) {
                viewer_state.closing_since = Some(current_time);
            }
        });

    keep_open
}

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
