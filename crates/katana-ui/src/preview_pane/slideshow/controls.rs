/* WHY: Specialized logic for slideshow navigation controls and close behavior to maintain modularity. */

use super::super::fullscreen::{
    FULLSCREEN_CLOSE_MARGIN, FULLSCREEN_CLOSE_SIZE, SLIDESHOW_BG_ALPHA_SCALE,
    SLIDESHOW_CONTROL_CORNER_RADIUS, SLIDESHOW_CONTROL_HEIGHT, SLIDESHOW_CONTROL_PADDING_X,
    SLIDESHOW_CONTROL_PADDING_Y, SLIDESHOW_CONTROL_SPACING, SLIDESHOW_CONTROL_WIDTH,
    SLIDESHOW_OPACITY_MIN,
};
use crate::icon::Icon;
use eframe::egui::{self, Vec2};

pub struct SlideshowControlsOps;

impl SlideshowControlsOps {
    pub fn render_slideshow_controls(
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        layout: &mut crate::state::layout::LayoutState,
        blocker_rect: egui::Rect,
        max_page: usize,
        opacity: f32,
    ) -> i32 {
        let control_rect = egui::Rect::from_center_size(
            egui::pos2(
                blocker_rect.center().x,
                blocker_rect.bottom() - FULLSCREEN_CLOSE_MARGIN - SLIDESHOW_CONTROL_HEIGHT / 2.0,
            ),
            egui::vec2(SLIDESHOW_CONTROL_WIDTH, SLIDESHOW_CONTROL_HEIGHT),
        );

        let mut page_delta: i32 = 0;
        if opacity > SLIDESHOW_OPACITY_MIN {
            ui.put(control_rect, |ui: &mut egui::Ui| {
                let mut bg = ui.visuals().window_fill();
                bg = bg.gamma_multiply(SLIDESHOW_BG_ALPHA_SCALE * opacity);
                egui::Frame::NONE
                    .fill(bg)
                    .corner_radius(SLIDESHOW_CONTROL_CORNER_RADIUS)
                    .inner_margin(egui::Margin::symmetric(
                        SLIDESHOW_CONTROL_PADDING_X,
                        SLIDESHOW_CONTROL_PADDING_Y,
                    ))
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            let icon_color = ui.visuals().text_color().gamma_multiply(opacity);
                            let mut prev_btn = egui::Button::image(
                                crate::icon::Icon::ChevronLeft
                                    .image(crate::icon::IconSize::Medium)
                                    .tint(icon_color),
                            )
                            .frame(false)
                            .fill(crate::theme_bridge::TRANSPARENT);
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
                            ui.label(egui::RichText::new(page_text).strong().color(icon_color));
                            ui.add_space(SLIDESHOW_CONTROL_SPACING);
                            let mut next_btn = egui::Button::image(
                                crate::icon::Icon::ChevronRight
                                    .image(crate::icon::IconSize::Medium)
                                    .tint(icon_color),
                            )
                            .frame(false)
                            .fill(crate::theme_bridge::TRANSPARENT);
                            if layout.slideshow_page >= max_page {
                                next_btn = next_btn.sense(egui::Sense::hover());
                            }
                            if ui.add(next_btn).clicked() && layout.slideshow_page < max_page {
                                page_delta = 1;
                            }
                        });
                    })
                    .response
            });
        }

        page_delta
    }

    pub fn render_slideshow_close_button(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        layout: &mut crate::state::layout::LayoutState,
        blocker_rect: egui::Rect,
        opacity: f32,
    ) {
        let msgs = crate::i18n::I18nOps::get();
        let close_btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);
        let close_btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                blocker_rect.right() - close_btn_size.x - FULLSCREEN_CLOSE_MARGIN,
                blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
            ),
            close_btn_size,
        );
        let text_color = ctx
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::BLACK, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.text)
            })
            .gamma_multiply(opacity);
        let close_resp = ui.put(
            close_btn_rect,
            egui::Button::image(
                Icon::CloseModal
                    .image(crate::icon::IconSize::Large)
                    .tint(text_color),
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
    }
}
