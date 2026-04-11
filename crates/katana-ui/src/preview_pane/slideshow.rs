use crate::icon::Icon;
use eframe::egui::{self, Vec2};

use super::fullscreen::{
    FULLSCREEN_CLOSE_MARGIN, FULLSCREEN_CLOSE_SIZE, FULLSCREEN_PADDING, SLIDESHOW_BG_ALPHA_SCALE,
    SLIDESHOW_CONTROL_CORNER_RADIUS, SLIDESHOW_CONTROL_FADE_DELAY, SLIDESHOW_CONTROL_FADE_DURATION,
    SLIDESHOW_CONTROL_HEIGHT, SLIDESHOW_CONTROL_PADDING_X, SLIDESHOW_CONTROL_PADDING_Y,
    SLIDESHOW_CONTROL_SPACING, SLIDESHOW_CONTROL_WIDTH, SLIDESHOW_OPACITY_MAX,
    SLIDESHOW_OPACITY_MIN,
};

pub(super) fn render_slideshow_modal(
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
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
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
            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp))
            {
                layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
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
                                pane.render_sections(ui, None, None, None, false, true);
                            },
                        );
                    });

                let content_height = out.content_size.y;
                let max_page = (content_height / viewport_height).floor() as usize;
                if layout.slideshow_page > max_page {
                    layout.slideshow_page = max_page;
                }

                let page_delta = render_slideshow_controls(ctx, ui, layout, blocker_rect, max_page);
                if page_delta < 0 {
                    layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
                } else if page_delta > 0 {
                    layout.slideshow_page += 1;
                }
            });

            render_slideshow_close_button(ctx, ui, layout, blocker_rect);
            render_slideshow_settings_sidebar(ctx, ui, layout, blocker_rect);
        });
}

fn render_slideshow_controls(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layout: &mut crate::state::layout::LayoutState,
    blocker_rect: egui::Rect,
    max_page: usize,
) -> i32 {
    let control_rect = egui::Rect::from_center_size(
        egui::pos2(
            blocker_rect.center().x,
            blocker_rect.bottom() - FULLSCREEN_CLOSE_MARGIN - SLIDESHOW_CONTROL_HEIGHT / 2.0,
        ),
        egui::vec2(SLIDESHOW_CONTROL_WIDTH, SLIDESHOW_CONTROL_HEIGHT),
    );
    let current_time = ctx.input(|i| i.time);
    let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
    let hover_controls = pointer_pos.is_some_and(|p| control_rect.contains(p));
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
        let progress =
            ((idle_time - SLIDESHOW_CONTROL_FADE_DELAY) as f32) / SLIDESHOW_CONTROL_FADE_DURATION;
        opacity =
            (SLIDESHOW_OPACITY_MAX - progress).clamp(SLIDESHOW_OPACITY_MIN, SLIDESHOW_OPACITY_MAX);
        if opacity > SLIDESHOW_OPACITY_MIN && opacity < SLIDESHOW_OPACITY_MAX {
            ctx.request_repaint();
        }
    }

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
                        let mut prev_btn = crate::icon::Icon::ChevronLeft
                            .button(ui, crate::icon::IconSize::Medium)
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
                        ui.label(egui::RichText::new(page_text).strong().color(icon_color));
                        ui.add_space(SLIDESHOW_CONTROL_SPACING);
                        let mut next_btn = crate::icon::Icon::ChevronRight
                            .button(ui, crate::icon::IconSize::Medium)
                            .frame(false);
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

fn render_slideshow_close_button(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layout: &mut crate::state::layout::LayoutState,
    blocker_rect: egui::Rect,
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
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        })
        .map_or(crate::theme_bridge::BLACK, |tc| {
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.text)
        });
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

fn render_slideshow_settings_sidebar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layout: &mut crate::state::layout::LayoutState,
    blocker_rect: egui::Rect,
) {
    const FULLSCREEN_INDICATOR_SPACING: f32 = 8.0;

    const SETTINGS_PANEL_WIDTH: f32 = 250.0;
    const SETTINGS_PADDING_TOP: f32 = 20.0;
    const SETTINGS_ITEM_SPACING: f32 = 12.0;

    let msgs = crate::i18n::I18nOps::get();
    let btn_size = Vec2::splat(FULLSCREEN_CLOSE_SIZE);

    let toggle_btn_rect = egui::Rect::from_min_size(
        egui::pos2(
            blocker_rect.right() - btn_size.x * 2.0 - FULLSCREEN_INDICATOR_SPACING,
            blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
        ),
        btn_size,
    );

    let text_color = ctx
        .data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        })
        .map_or(crate::theme_bridge::BLACK, |tc| {
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.text)
        });

    let toggle_resp = ui.put(
        toggle_btn_rect,
        egui::Button::image(
            Icon::Settings
                .image(crate::icon::IconSize::Large)
                .tint(text_color),
        )
        .fill(crate::theme_bridge::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, crate::theme_bridge::TRANSPARENT)),
    );

    if toggle_resp
        .on_hover_text(&msgs.preview.slideshow_settings)
        .clicked()
    {
        layout.slideshow_settings_open = !layout.slideshow_settings_open;
    }

    if layout.slideshow_settings_open {
        let panel_width = SETTINGS_PANEL_WIDTH;
        let panel_rect = egui::Rect::from_min_size(
            egui::pos2(blocker_rect.right() - panel_width, blocker_rect.top()),
            egui::vec2(panel_width, blocker_rect.height()),
        );

        let bg_color = ctx
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.panel_background)
            });

        ui.painter().rect_filled(panel_rect, 0.0, bg_color);
        ui.painter().line_segment(
            [panel_rect.left_top(), panel_rect.left_bottom()],
            ui.visuals().window_stroke(),
        );

        ui.put(panel_rect, |ui: &mut egui::Ui| {
            egui::Frame::NONE
                .inner_margin(FULLSCREEN_PADDING)
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.heading(&msgs.preview.slideshow_settings);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::image(
                                        Icon::CloseModal
                                            .image(crate::icon::IconSize::Medium)
                                            .tint(text_color),
                                    )
                                    .fill(crate::theme_bridge::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE),
                                )
                                .clicked()
                            {
                                layout.slideshow_settings_open = false;
                            }
                        });
                    });

                    ui.add_space(SETTINGS_PADDING_TOP);

                    if ui
                        .add(
                            crate::widgets::LabeledToggle::new(
                                &msgs.preview.highlight_hover,
                                &mut layout.slideshow_hover_highlight,
                            )
                            .position(crate::widgets::TogglePosition::Right)
                            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                        )
                        .changed()
                    {
                        ctx.request_repaint();
                    }

                    ui.add_space(SETTINGS_ITEM_SPACING);

                    if ui
                        .add(
                            crate::widgets::LabeledToggle::new(
                                &msgs.preview.show_diagram_controls,
                                &mut layout.slideshow_show_diagram_controls,
                            )
                            .position(crate::widgets::TogglePosition::Right)
                            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                        )
                        .changed()
                    {
                        ctx.request_repaint();
                    }
                })
                .response
        });
    }
}
