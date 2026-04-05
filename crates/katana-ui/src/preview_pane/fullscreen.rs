use crate::icon::Icon;
use crate::preview_pane::{RenderedSection, ViewerState};
use eframe::egui::{self, Vec2};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const FULLSCREEN_PADDING: f32 = 40.0;
pub(crate) const FULLSCREEN_CLOSE_SIZE: f32 = 32.0;
pub(crate) const FULLSCREEN_CLOSE_MARGIN: f32 = 20.0;
pub(crate) const MIN_ZOOM: f32 = 0.5;
pub(crate) const MAX_ZOOM: f32 = 5.0;

const SLIDESHOW_CONTROL_HEIGHT: f32 = 36.0;
const SLIDESHOW_CONTROL_WIDTH: f32 = 160.0;
const SLIDESHOW_CONTROL_CORNER_RADIUS: f32 = 18.0;
const SLIDESHOW_CONTROL_PADDING_X: i8 = 20;
const SLIDESHOW_CONTROL_PADDING_Y: i8 = 4;
const SLIDESHOW_CONTROL_SPACING: f32 = 8.0;
const SLIDESHOW_BG_ALPHA_SCALE: f32 = 0.8;

const SLIDESHOW_CONTROL_FADE_DELAY: f64 = 1.5;
const SLIDESHOW_CONTROL_FADE_DURATION: f32 = 0.5;
const SLIDESHOW_OPACITY_MAX: f32 = 1.0;
const SLIDESHOW_OPACITY_MIN: f32 = 0.0;

pub use super::types::FullscreenLogicOps;

impl FullscreenLogicOps {
    pub(crate) fn render_fullscreen_if_active(
        ctx: &egui::Context,
        sections: &[RenderedSection],
        fullscreen_image: Option<usize>,
        fullscreen_state: &mut ViewerState,
    ) -> Option<usize> {
        let idx = fullscreen_image?;
        if let Some(RenderedSection::Image { svg_data, alt, .. }) = sections.get(idx) {
            if Self::show_fullscreen_modal(ctx, svg_data, alt, fullscreen_state, idx) {
                Some(idx) /* WHY: keep open */
            } else {
                None /* WHY: user closed */
            }
        } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
            if Self::show_fullscreen_local_image(ctx, path, alt, fullscreen_state, idx) {
                Some(idx) /* WHY: keep open */
            } else {
                None /* WHY: user closed */
            }
        } else {
            None /* WHY: section gone */
        }
    }

    pub(crate) fn show_fullscreen_modal(
        ctx: &egui::Context,
        img: &RasterizedSvg,
        _alt: &str,
        viewer_state: &mut ViewerState,
        idx: usize,
    ) -> bool {
        let msgs = crate::i18n::I18nOps::get();
        let dc = &msgs.preview.diagram_controller;

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return false;
        }

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
                        viewer_state.zoom =
                            (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                    }
                    if response.dragged() {
                        viewer_state.pan += response.drag_delta();
                    } else {
                        viewer_state.pan += ui.input(|i| i.smooth_scroll_delta);
                    }
                }

                let bg_color = crate::theme_bridge::IMAGE_VIEWER_OVERLAY_COLOR;
                ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

                let avail = Vec2::new(
                    screen.width() - FULLSCREEN_PADDING * 2.0,
                    screen.height() - FULLSCREEN_PADDING * 2.0,
                );
                let scale_x = avail.x / img.width as f32;
                let scale_y = avail.y / img.height as f32;
                let base_scale = scale_x.min(scale_y).min(1.0);
                let zoom = viewer_state.zoom;
                let pan = viewer_state.pan;
                let size = Vec2::new(
                    img.width as f32 * base_scale * zoom,
                    img.height as f32 * base_scale * zoom,
                );
                let texture_handle = if viewer_state.texture.is_none() {
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
                    let th = ctx.load_texture(
                        format!("diagram_fs_{idx}"),
                        color_img,
                        egui::TextureOptions::LINEAR,
                    );
                    viewer_state.texture = Some(th.clone());
                    th
                } else {
                    viewer_state.texture.clone().unwrap()
                };

                let img_pos = screen.center() - size / 2.0 + pan;
                let img_rect = egui::Rect::from_min_size(img_pos, size);
                ui.painter().with_clip_rect(blocker_rect).image(
                    texture_handle.id(),
                    img_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    crate::theme_bridge::WHITE,
                );

                crate::diagram_controller::DiagramControllerOps::draw_controls(
                    ui,
                    viewer_state,
                    blocker_rect,
                );

                let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
                let close_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                        blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
                    ),
                    close_btn_size,
                );
                let close_resp = ui.put(
                    close_btn_rect,
                    egui::Button::image(
                        Icon::CloseModal
                            .image(crate::icon::IconSize::Large)
                            .tint(crate::theme_bridge::WHITE),
                    )
                    .fill(
                        crate::theme_bridge::TRANSPARENT, /* WHY: Handled by theme overlay */
                    )
                    .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
                );
                if close_resp.on_hover_text(&dc.close).clicked() {
                    keep_open = false;
                }
            });

        keep_open
    }

    pub(crate) fn show_fullscreen_local_image(
        ctx: &egui::Context,
        path: &std::path::Path,
        _alt: &str,
        viewer_state: &mut ViewerState,
        idx: usize,
    ) -> bool {
        let msgs = crate::i18n::I18nOps::get();
        let dc = &msgs.preview.diagram_controller;

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return false;
        }

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
                        viewer_state.zoom =
                            (viewer_state.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                    }
                    if response.dragged() {
                        viewer_state.pan += response.drag_delta();
                    } else {
                        viewer_state.pan += ui.input(|i| i.smooth_scroll_delta);
                    }
                }

                let bg_color = crate::theme_bridge::IMAGE_VIEWER_OVERLAY_COLOR;
                ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

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
                        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
                        viewer_state.texture = Some(ui.ctx().load_texture(
                            format!("local_image_fs_{idx}"),
                            color_img,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                    viewer_state.texture.clone()
                } else {
                    viewer_state.texture.clone()
                };

                let (texture_handle, width, height) = match texture_handle {
                    Some(t) => {
                        let size = t.size();
                        (t, size[0], size[1])
                    }
                    None => return,
                };

                let avail = Vec2::new(
                    screen.width() - FULLSCREEN_PADDING * 2.0,
                    screen.height() - FULLSCREEN_PADDING * 2.0,
                );
                let scale_x = avail.x / width as f32;
                let scale_y = avail.y / height as f32;
                let base_scale = scale_x.min(scale_y).min(1.0);

                let zoom = viewer_state.zoom;
                let pan = viewer_state.pan;
                let size = Vec2::new(
                    width as f32 * base_scale * zoom,
                    height as f32 * base_scale * zoom,
                );

                let img_pos = screen.center() - size / 2.0 + pan;
                let img_rect = egui::Rect::from_min_size(img_pos, size);
                ui.painter().with_clip_rect(blocker_rect).image(
                    texture_handle.id(),
                    img_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    crate::theme_bridge::WHITE,
                );

                crate::diagram_controller::DiagramControllerOps::draw_controls(
                    ui,
                    viewer_state,
                    blocker_rect,
                );

                let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
                let close_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                        blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
                    ),
                    close_btn_size,
                );

                let close_resp = ui.put(
                    close_btn_rect,
                    egui::Button::image(
                        crate::icon::Icon::CloseModal
                            .image(crate::icon::IconSize::Large)
                            .tint(crate::theme_bridge::WHITE),
                    )
                    .fill(
                        crate::theme_bridge::TRANSPARENT, /* WHY: Handled by theme overlay */
                    )
                    .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
                );

                if close_resp.on_hover_text(&dc.close).clicked() {
                    keep_open = false;
                }
            });

        keep_open
    }

    pub(crate) fn render_slideshow_modal(
        ctx: &egui::Context,
        layout: &mut crate::state::layout::LayoutState,
        pane: &mut crate::preview_pane::PreviewPane,
    ) {
        if !layout.show_slideshow {
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            layout.show_slideshow = false;
            if !layout.was_os_fullscreen_before_slideshow {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            return;
        }

        let screen = ctx.content_rect();
        let bg_color = ctx
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.background)
            });

        egui::Area::new(egui::Id::new("slideshow_modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (blocker_rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::hover());
                ui.painter().rect_filled(blocker_rect, 0.0, bg_color);

                let content_rect = blocker_rect.shrink(FULLSCREEN_PADDING);

                /* WHY: Handle keyboard paging */
                if ui.input(|i| {
                    i.key_pressed(egui::Key::ArrowRight)
                        || i.key_pressed(egui::Key::PageDown)
                        || i.key_pressed(egui::Key::Space)
                }) {
                    layout.slideshow_page += 1;
                }
                if ui.input(|i| {
                    i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp)
                }) {
                    layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
                }

                ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    let viewport_height = content_rect.height();
                    let target_offset = layout.slideshow_page as f32 * viewport_height;

                    let out = egui::ScrollArea::vertical()
                        /* WHY: Explicitly manage the exact offset to enforce rigid page scrolling */
                        .vertical_scroll_offset(target_offset)
                        .auto_shrink([false; 2])
                        /* WHY: hide the scroll bar to make it feel like a real slideshow viewer */
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            let inner_width = ui.available_width();
                            let child_rect = egui::Rect::from_min_size(
                                ui.next_widget_position(),
                                egui::vec2(inner_width, 0.0),
                            );
                            ui.scope_builder(
                                egui::UiBuilder::new()
                                    .max_rect(child_rect)
                                    .layout(egui::Layout::top_down(egui::Align::Min)),
                                |ui| {
                                    pane.render_sections(ui, None, None, None, false, true);
                                },
                            );
                        });

                    let content_height = out.content_size.y;
                    let max_page = (content_height / viewport_height).floor() as usize;

                    if layout.slideshow_page > max_page {
                        layout.slideshow_page = max_page;
                    }

                    /* WHY: Control block */
                    let control_rect = egui::Rect::from_center_size(
                        egui::pos2(
                            blocker_rect.center().x,
                            blocker_rect.bottom()
                                - FULLSCREEN_CLOSE_MARGIN
                                - SLIDESHOW_CONTROL_HEIGHT / 2.0,
                        ),
                        egui::vec2(SLIDESHOW_CONTROL_WIDTH, SLIDESHOW_CONTROL_HEIGHT),
                    );

                    let current_time = ctx.input(|i| i.time);
                    let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
                    let mut hover_controls = false;

                    if let Some(pos) = pointer_pos
                        && control_rect.contains(pos)
                    {
                        hover_controls = true;
                    }

                    let is_active = ctx.input(|i| {
                        hover_controls
                            || i.pointer.velocity() != egui::Vec2::ZERO
                            || i.pointer.any_pressed()
                            || !i.events.is_empty()
                    });

                    if is_active {
                        layout.slideshow_last_active_time = current_time;
                    }

                    let idle_time = current_time - layout.slideshow_last_active_time;

                    let mut opacity = SLIDESHOW_OPACITY_MAX;
                    if idle_time > SLIDESHOW_CONTROL_FADE_DELAY {
                        let progress = ((idle_time - SLIDESHOW_CONTROL_FADE_DELAY) as f32)
                            / SLIDESHOW_CONTROL_FADE_DURATION;
                        opacity = (SLIDESHOW_OPACITY_MAX - progress)
                            .clamp(SLIDESHOW_OPACITY_MIN, SLIDESHOW_OPACITY_MAX);
                        if opacity > SLIDESHOW_OPACITY_MIN && opacity < SLIDESHOW_OPACITY_MAX {
                            ctx.request_repaint();
                        }
                    }

                    let mut page_delta: i32 = 0;

                    if opacity > SLIDESHOW_OPACITY_MIN {
                        ui.put(control_rect, |ui: &mut egui::Ui| {
                            let mut bg = ui.visuals().window_fill();
                            bg = bg.gamma_multiply(SLIDESHOW_BG_ALPHA_SCALE * opacity);

                            let frame = egui::Frame::NONE
                                .fill(bg)
                                .corner_radius(SLIDESHOW_CONTROL_CORNER_RADIUS)
                                .inner_margin(egui::Margin::symmetric(
                                    SLIDESHOW_CONTROL_PADDING_X,
                                    SLIDESHOW_CONTROL_PADDING_Y,
                                ));

                            frame
                                .show(ui, |ui| {
                                    ui.horizontal_centered(|ui| {
                                        let mut icon_color = ui.visuals().text_color();
                                        icon_color = icon_color.gamma_multiply(opacity);

                                        /* WHY: allow(icon_button_fill) */
                                        let mut prev_btn = egui::Button::image(
                                            crate::icon::Icon::ChevronLeft
                                                .image(crate::icon::IconSize::Medium)
                                                .tint(icon_color),
                                        )
                                        .fill(if ui.visuals().dark_mode {
                                            crate::theme_bridge::TRANSPARENT
                                        } else {
                                            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg(
                                            )
                                        })
                                        .frame(false);
                                        if layout.slideshow_page == 0 {
                                            prev_btn = prev_btn.sense(egui::Sense::hover());
                                        }

                                        if ui.add(prev_btn).clicked() && layout.slideshow_page > 0 {
                                            page_delta = -1;
                                        }

                                        ui.add_space(SLIDESHOW_CONTROL_SPACING);

                                        let page_text = [
                                            (layout.slideshow_page + 1).to_string(),
                                            (max_page + 1).to_string(),
                                        ]
                                        .join(" / ");

                                        ui.label(
                                            egui::RichText::new(page_text)
                                                .strong()
                                                .color(icon_color),
                                        );

                                        ui.add_space(SLIDESHOW_CONTROL_SPACING);

                                        /* WHY: allow(icon_button_fill) */
                                        let mut next_btn = egui::Button::image(
                                            crate::icon::Icon::ChevronRight
                                                .image(crate::icon::IconSize::Medium)
                                                .tint(icon_color),
                                        )
                                        .fill(if ui.visuals().dark_mode {
                                            crate::theme_bridge::TRANSPARENT
                                        } else {
                                            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg(
                                            )
                                        })
                                        .frame(false);
                                        if layout.slideshow_page >= max_page {
                                            next_btn = next_btn.sense(egui::Sense::hover());
                                        }

                                        if ui.add(next_btn).clicked()
                                            && layout.slideshow_page < max_page
                                        {
                                            page_delta = 1;
                                        }
                                    });
                                })
                                .response
                        });
                    }

                    if page_delta < 0 {
                        layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
                    } else if page_delta > 0 {
                        layout.slideshow_page += 1;
                    }
                });

                /* WHY: Close button */
                let msgs = crate::i18n::I18nOps::get();
                let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
                let close_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                        blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
                    ),
                    close_btn_size,
                );

                let close_resp = ui.put(
                    close_btn_rect,
                    egui::Button::image(
                        Icon::CloseModal.image(crate::icon::IconSize::Large).tint(
                            ctx.data(|d| {
                                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                    "katana_theme_colors",
                                ))
                            })
                            .map_or(
                                crate::theme_bridge::BLACK,
                                |tc| {
                                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                        tc.preview.text,
                                    )
                                },
                            ),
                        ),
                    )
                    .fill(crate::theme_bridge::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
                );

                if close_resp
                    .on_hover_text(&msgs.preview.diagram_controller.close)
                    .clicked()
                {
                    layout.show_slideshow = false;
                    if !layout.was_os_fullscreen_before_slideshow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    }
                }
            });
    }
}
