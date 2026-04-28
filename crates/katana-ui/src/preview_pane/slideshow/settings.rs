/* WHY: Encapsulated sidebar settings logic for better maintainability and state isolation. */

use super::super::fullscreen::FULLSCREEN_PADDING;
use eframe::egui;

pub const SETTINGS_PANEL_WIDTH: f32 = 250.0;
pub const SETTINGS_PADDING_TOP: f32 = 20.0;
pub const SETTINGS_ITEM_SPACING: f32 = 12.0;
pub const SETTINGS_TAB_WIDTH: f32 = 24.0;
pub const SETTINGS_TAB_HEIGHT: f32 = 48.0;
pub const SETTINGS_TAB_RADIUS: u8 = 8;
pub const SETTINGS_BORDER_WIDTH: f32 = 1.5;

pub struct SlideshowSettingsOps;

impl SlideshowSettingsOps {
    pub fn render_slideshow_settings_sidebar(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        layout: &mut crate::state::layout::LayoutState,
        blocker_rect: egui::Rect,
        opacity: f32,
    ) {
        let open_factor = ctx.animate_bool(
            egui::Id::new("slideshow_settings_anim"),
            layout.slideshow_settings_open,
        );

        let panel_width = SETTINGS_PANEL_WIDTH;
        let panel_x = blocker_rect.right() - (panel_width * open_factor);
        let panel_rect = egui::Rect::from_min_size(
            egui::pos2(panel_x, blocker_rect.top()),
            egui::vec2(panel_width, blocker_rect.height()),
        );

        let outrigger_size = egui::vec2(SETTINGS_TAB_WIDTH, SETTINGS_TAB_HEIGHT);
        let toggle_btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                panel_x - outrigger_size.x,
                blocker_rect.center().y - outrigger_size.y / 2.0,
            ),
            outrigger_size,
        );
        Self::consume_slideshow_settings_sidebar_input(
            ui,
            panel_rect,
            toggle_btn_rect,
            open_factor,
        );

        let msgs = crate::i18n::I18nOps::get();
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

        let bg_color = ctx
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.panel_background)
            })
            .gamma_multiply(opacity);

        let tab_rounding = egui::CornerRadius {
            nw: SETTINGS_TAB_RADIUS,
            sw: SETTINGS_TAB_RADIUS,
            ne: 0,
            se: 0,
        };
        ui.painter()
            .rect_filled(toggle_btn_rect, tab_rounding, bg_color);
        if open_factor > 0.0 {
            ui.painter().line_segment(
                [toggle_btn_rect.right_top(), toggle_btn_rect.right_bottom()],
                egui::Stroke::new(SETTINGS_BORDER_WIDTH, bg_color),
            );
        }

        let toggle_icon = if layout.slideshow_settings_open {
            crate::icon::Icon::ChevronRight
        } else {
            crate::icon::Icon::Settings
        };

        let toggle_resp = ui.put(
            toggle_btn_rect,
            egui::Button::image(
                toggle_icon
                    .image(crate::icon::IconSize::Medium)
                    .tint(text_color),
            )
            .fill(crate::theme_bridge::TRANSPARENT)
            .frame(false),
        );

        if toggle_resp
            .on_hover_text(&msgs.preview.slideshow_settings)
            .clicked()
        {
            layout.slideshow_settings_open = !layout.slideshow_settings_open;
        }

        if open_factor > 0.0 {
            ui.painter().rect_filled(panel_rect, 0.0, bg_color);
            ui.painter().line_segment(
                [panel_rect.left_top(), panel_rect.left_bottom()],
                ui.visuals().window_stroke(),
            );

            let mut child_ui =
                ui.child_ui(panel_rect, egui::Layout::top_down(egui::Align::Min), None);
            egui::Frame::NONE
                .inner_margin(FULLSCREEN_PADDING)
                .show(&mut child_ui, |ui| {
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
                });
        }
    }

    fn consume_slideshow_settings_sidebar_input(
        ui: &mut egui::Ui,
        panel_rect: egui::Rect,
        toggle_btn_rect: egui::Rect,
        open_factor: f32,
    ) {
        let consume_rect = if open_factor > 0.0 {
            panel_rect.union(toggle_btn_rect)
        } else {
            toggle_btn_rect
        };

        crate::widgets::InteractionFacade::consume_rect(
            ui,
            "slideshow_settings_sidebar_input_blocker",
            consume_rect,
        );
    }
}
