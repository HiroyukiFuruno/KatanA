/* WHY: Specialized logic for the main slideshow modal to satisfy file length limits and isolate presentation state. */

use super::super::fullscreen::{
    FULLSCREEN_PADDING, SLIDESHOW_CONTROL_FADE_DELAY, SLIDESHOW_CONTROL_FADE_DURATION,
    SLIDESHOW_OPACITY_MAX, SLIDESHOW_OPACITY_MIN,
};
use super::controls::SlideshowControlsOps;
use super::settings::SlideshowSettingsOps;
use eframe::egui;

pub struct SlideshowModalOps;

impl SlideshowModalOps {
    pub fn render_slideshow_modal(
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

                let current_time = ctx.input(|i| i.time);
                let is_active = ctx.input(|i| {
                    i.pointer.velocity() != egui::Vec2::ZERO
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

                ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    let viewport_height = content_rect.height();
                    let target_offset = layout.slideshow_page as f32 * viewport_height;
                    let out = egui::ScrollArea::vertical()
                        .vertical_scroll_offset(target_offset)
                        .auto_shrink([false; 2])
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
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(
                                            egui::Id::new("katana_slideshow_hover_highlight"),
                                            layout.slideshow_hover_highlight,
                                        );
                                        d.insert_temp(
                                            egui::Id::new("katana_slideshow_diagram_controls"),
                                            layout.slideshow_show_diagram_controls,
                                        );
                                    });
                                    pane.render_sections(ui, None, None, None, None, true);
                                },
                            );
                        });

                    let content_height = out.content_size.y;
                    let max_page = (content_height / viewport_height).floor() as usize;
                    if layout.slideshow_page > max_page {
                        layout.slideshow_page = max_page;
                    }

                    let page_delta = SlideshowControlsOps::render_slideshow_controls(
                        ctx,
                        ui,
                        layout,
                        blocker_rect,
                        max_page,
                        opacity,
                    );
                    if page_delta < 0 {
                        layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
                    } else if page_delta > 0 {
                        layout.slideshow_page += 1;
                    }
                });

                if opacity > SLIDESHOW_OPACITY_MIN {
                    SlideshowControlsOps::render_slideshow_close_button(
                        ctx,
                        ui,
                        layout,
                        blocker_rect,
                        opacity,
                    );
                    SlideshowSettingsOps::render_slideshow_settings_sidebar(
                        ctx,
                        ui,
                        layout,
                        blocker_rect,
                        opacity,
                    );
                }
            });
    }
}
