use crate::i18n;
use crate::icon::{Icon, IconSize};
use crate::preview_pane::ViewerState;
use crate::theme_bridge::ThemeBridgeOps;
use eframe::egui::{self, Shape, Vec2};

const BUTTON_SIZE: f32 = 28.0;
const MARGIN: f32 = 8.0;
const GAP: f32 = 2.0;
const GRID_DIM: f32 = 3.0;
const CORNER_RADIUS: u8 = 6;
const CONTROL_ICON_ALPHA: u8 = 0xf0;
const CONTROL_ICON_HOVER_ALPHA: u8 = 0xff;
const CONTROL_BG_ALPHA: u8 = 0x95;
const CONTROL_BG_HOVER_ALPHA: u8 = 0xbb;
const CONTROL_BG_ACTIVE_ALPHA: u8 = 0xe0;
const CONTROL_STROKE_ALPHA: u8 = 0x55;
const CONTROL_STROKE_HOVER_ALPHA: u8 = 0x9a;
const CONTROL_STROKE_FOCUS_ALPHA: u8 = 0xe0;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ControlAction {
    None,
}

pub(crate) struct DiagramControllerOps;

impl DiagramControllerOps {
    fn control_icon_color(is_hovered: bool, is_active: bool) -> egui::Color32 {
        if is_active || is_hovered {
            ThemeBridgeOps::from_white_alpha(CONTROL_ICON_HOVER_ALPHA)
        } else {
            ThemeBridgeOps::from_white_alpha(CONTROL_ICON_ALPHA)
        }
    }

    fn control_background_color(is_hovered: bool, is_active: bool) -> egui::Color32 {
        if is_active {
            ThemeBridgeOps::from_black_alpha(CONTROL_BG_ACTIVE_ALPHA)
        } else if is_hovered {
            ThemeBridgeOps::from_black_alpha(CONTROL_BG_HOVER_ALPHA)
        } else {
            ThemeBridgeOps::from_black_alpha(CONTROL_BG_ALPHA)
        }
    }

    fn control_stroke(is_hovered: bool, is_active: bool, has_focus: bool) -> egui::Stroke {
        if has_focus {
            egui::Stroke::new(
                1.0,
                ThemeBridgeOps::from_white_alpha(CONTROL_STROKE_FOCUS_ALPHA),
            )
        } else if is_active || is_hovered {
            egui::Stroke::new(
                1.0,
                ThemeBridgeOps::from_white_alpha(CONTROL_STROKE_HOVER_ALPHA),
            )
        } else {
            egui::Stroke::new(1.0, ThemeBridgeOps::from_white_alpha(CONTROL_STROKE_ALPHA))
        }
    }

    fn render_control_button(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        icon: Icon,
        tooltip: &str,
        clickable: bool,
    ) -> egui::Response {
        let (is_hovered, is_active) = ui.input(|i| {
            let hovered = i.pointer.hover_pos().is_some_and(|pos| rect.contains(pos));
            let active = i.pointer.primary_down() && hovered;
            (hovered, active)
        });

        let response = if clickable {
            ui.put(
                rect,
                egui::Button::image(
                    icon.image(IconSize::Large)
                        .tint(Self::control_icon_color(is_hovered, is_active)),
                )
                .fill(Self::control_background_color(is_hovered, is_active))
                .stroke(Self::control_stroke(is_hovered, is_active, false)),
            )
        } else {
            ui.put(
                rect,
                egui::Button::image(
                    icon.image(IconSize::Large)
                        .tint(Self::control_icon_color(is_hovered, false)),
                )
                .fill(Self::control_background_color(is_hovered, false))
                .stroke(Self::control_stroke(is_hovered, false, false)),
            )
        };

        let has_focus = response.has_focus();
        if has_focus {
            ui.painter().add(Shape::rect_stroke(
                rect,
                egui::CornerRadius::same(CORNER_RADIUS),
                Self::control_stroke(is_hovered, is_active, true),
                egui::StrokeKind::Inside,
            ));
        }

        response.on_hover_text(tooltip)
    }

    pub(crate) fn draw_controls(
        ui: &mut egui::Ui,
        state: &mut ViewerState,
        container_rect: egui::Rect,
    ) -> ControlAction {
        let msgs = i18n::I18nOps::get();
        let dc = &msgs.preview.diagram_controller;

        let grid_w = BUTTON_SIZE * GRID_DIM + GAP * (GRID_DIM - 1.0);
        let grid_h = BUTTON_SIZE * GRID_DIM + GAP * (GRID_DIM - 1.0);
        let grid_origin = egui::pos2(
            container_rect.right() - grid_w - MARGIN,
            container_rect.bottom() - grid_h - MARGIN,
        );

        let btn_rect = |col: f32, row: f32| -> egui::Rect {
            egui::Rect::from_min_size(
                egui::pos2(
                    grid_origin.x + col * (BUTTON_SIZE + GAP),
                    grid_origin.y + row * (BUTTON_SIZE + GAP),
                ),
                Vec2::splat(BUTTON_SIZE),
            )
        };

        if Self::render_control_button(ui, btn_rect(1.0, 0.0), Icon::PanUp, &dc.pan_up, true)
            .clicked()
        {
            state.pan_up();
        }
        if Self::render_control_button(ui, btn_rect(2.0, 0.0), Icon::ZoomIn, &dc.zoom_in, true)
            .clicked()
        {
            state.zoom_in();
        }

        if Self::render_control_button(ui, btn_rect(0.0, 1.0), Icon::PanLeft, &dc.pan_left, true)
            .clicked()
        {
            state.pan_left();
        }
        if Self::render_control_button(ui, btn_rect(1.0, 1.0), Icon::ResetView, &dc.reset, true)
            .clicked()
        {
            state.reset();
        }
        if Self::render_control_button(ui, btn_rect(2.0, 1.0), Icon::PanRight, &dc.pan_right, true)
            .clicked()
        {
            state.pan_right();
        }

        Self::render_control_button(ui, btn_rect(0.0, 2.0), Icon::Info, &dc.trackpad_help, false);

        if Self::render_control_button(ui, btn_rect(1.0, 2.0), Icon::PanDown, &dc.pan_down, true)
            .clicked()
        {
            state.pan_down();
        }
        if Self::render_control_button(ui, btn_rect(2.0, 2.0), Icon::ZoomOut, &dc.zoom_out, true)
            .clicked()
        {
            state.zoom_out();
        }

        ControlAction::None
    }

    pub(crate) fn draw_fullscreen_button(ui: &mut egui::Ui, container_rect: egui::Rect) -> bool {
        let msgs = i18n::I18nOps::get();
        let dc = &msgs.preview.diagram_controller;

        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                container_rect.right() - BUTTON_SIZE - MARGIN,
                container_rect.top() + MARGIN,
            ),
            Vec2::splat(BUTTON_SIZE),
        );
        Self::render_control_button(ui, btn_rect, Icon::Fullscreen, &dc.fullscreen, true).clicked()
    }
}

#[cfg(test)]
mod tests {
    use super::DiagramControllerOps;

    #[test]
    fn diagram_overlay_controls_use_non_theme_overlay_tokens() {
        let idle = DiagramControllerOps::control_background_color(false, false);
        let hover = DiagramControllerOps::control_background_color(true, false);
        let active = DiagramControllerOps::control_background_color(false, true);
        let icon = DiagramControllerOps::control_icon_color(false, false);
        let focus = DiagramControllerOps::control_stroke(false, false, true);

        assert!(idle.a() >= 0x90);
        assert!(hover.a() > idle.a());
        assert!(active.a() > hover.a());
        assert!(icon.a() >= 0xf0);
        assert!(focus.color.a() >= 0xe0);
    }
}
