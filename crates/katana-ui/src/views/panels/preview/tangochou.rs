use crate::app_state::ViewMode;
use crate::theme_bridge::{ThemeBridgeOps, WHITE};
use eframe::egui;

use super::tangochou_consts::*;

/* WHY: Flashcard-style widget for toggling between Code/Preview modes.
Named after Japanese "単語帳 (tangochou)" flash-card notebooks. */
pub(super) struct TangochouWidget<'a> {
    pub view_mode: ViewMode,
    pub label_front: &'a str,
    pub label_back: &'a str,
}
impl<'a> TangochouWidget<'a> {
    /* WHY: Returns Some(action) when clicked so caller owns the dispatch. */
    pub fn show(self, ui: &mut egui::Ui) -> Option<crate::app_state::AppAction> {
        let is_code = self.view_mode == ViewMode::CodeOnly;

        let total_w = ui.available_width();
        let card_w = total_w - CARD_PADDING * 2.0;

        let base_pos = ui.cursor().min + egui::vec2(CARD_PADDING, CARD_TOP_OFFSET);
        let ring_center = base_pos + egui::vec2(RING_X_OFFSET, CARD_HEIGHT * CARD_CENTER_RATIO);

        let back_angle = CARD_BACK_ANGLE_DEG.to_radians();
        let drop_y = (card_w - RING_X_OFFSET) * back_angle.sin();
        let needed_h = CARD_HEIGHT + drop_y + CARD_EXTRA_H_PAD;

        let (_alloc_rect, response) =
            ui.allocate_exact_size(egui::vec2(total_w, needed_h), egui::Sense::click());

        let (actual_back_angle, actual_front_angle) = if response.hovered() {
            (
                back_angle + CARD_HOVER_BACK_EXTRA_DEG.to_radians(),
                -CARD_HOVER_FRONT_OFFSET_DEG.to_radians(),
            )
        } else {
            (back_angle, 0.0f32)
        };

        let is_dark = ui.visuals().dark_mode;
        let back_bg = ThemeBridgeOps::from_gray(if is_dark {
            CARD_GRAY_DARK_BACK
        } else {
            CARD_GRAY_LIGHT_BACK
        });
        let front_bg = ThemeBridgeOps::from_gray(if is_dark {
            CARD_GRAY_DARK_FRONT
        } else {
            CARD_GRAY_LIGHT_FRONT
        });
        let stroke_col = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let selection_col = ui.visuals().selection.bg_fill;

        let draw_card = |angle: f32, bg: egui::Color32, text: &str, is_front: bool| {
            let rect = egui::Rect::from_min_size(base_pos, egui::vec2(card_w, CARD_HEIGHT));
            let rotate_pos = |p: egui::Pos2| -> egui::Pos2 {
                let dx = p.x - ring_center.x;
                let dy = p.y - ring_center.y;
                let (sin_a, cos_a) = angle.sin_cos();
                egui::Pos2::new(
                    ring_center.x + dx * cos_a - dy * sin_a,
                    ring_center.y + dx * sin_a + dy * cos_a,
                )
            };

            let corners = [
                rect.left_top(),
                rect.right_top(),
                rect.right_bottom(),
                rect.left_bottom(),
            ]
            .map(rotate_pos);

            if is_front {
                ui.painter().add(egui::Shape::convex_polygon(
                    corners.map(|p| p + SHADOW_OFFSET).into(),
                    ThemeBridgeOps::from_black_alpha(SHADOW_ALPHA),
                    egui::Stroke::NONE,
                ));
            }

            let border_w = match is_front {
                true => FRONT_BORDER_W,
                false => BACK_BORDER_W,
            };
            let border_col = match is_front {
                true => selection_col,
                false => stroke_col,
            };
            ui.painter().add(egui::Shape::convex_polygon(
                corners.into(),
                bg,
                egui::Stroke::new(border_w, border_col),
            ));

            let hole_pos = rotate_pos(ring_center);
            ui.painter()
                .circle_filled(hole_pos, HOLE_RADIUS, ui.visuals().window_fill());
            ui.painter().circle_stroke(
                hole_pos,
                HOLE_RADIUS,
                egui::Stroke::new(BACK_BORDER_W, stroke_col),
            );

            let text_color = if is_front {
                ui.visuals().text_color()
            } else {
                ui.visuals()
                    .text_color()
                    .linear_multiply(HIGHLIGHT_ALPHA_MUL)
            };
            let galley = ui.painter().layout_no_wrap(
                text.to_owned(),
                egui::FontId::proportional(TEXT_SIZE),
                text_color,
            );
            let text_center = rect.center() + egui::vec2(TEXT_X_OFFSET, 0.0);
            let text_tl = text_center - galley.size() / 2.0;

            let mut shape = egui::epaint::TextShape::new(rotate_pos(text_tl), galley, text_color);
            shape.angle = angle;
            ui.painter().add(egui::Shape::Text(shape));
        };

        let (front_text, back_text) = if is_code {
            (self.label_front, self.label_back)
        } else {
            (self.label_back, self.label_front)
        };
        draw_card(actual_back_angle, back_bg, back_text, false);
        draw_card(actual_front_angle, front_bg, front_text, true);

        /* WHY: Layered ring strokes give a 3-D metallic binder-ring appearance. */
        let ring_color = ThemeBridgeOps::from_gray(if is_dark {
            RING_GRAY_DARK
        } else {
            RING_GRAY_LIGHT
        });
        let ring_shadow = ThemeBridgeOps::from_black_alpha(RING_ALPHA);
        ui.painter().circle_stroke(
            ring_center,
            RING_RADIUS,
            egui::Stroke::new(RING_STROKE_MAIN, ring_color),
        );
        ui.painter().circle_stroke(
            ring_center,
            RING_RADIUS_OUTER,
            egui::Stroke::new(RING_STROKE_ACCENT, ring_shadow),
        );
        ui.painter().circle_stroke(
            ring_center,
            RING_RADIUS_INNER,
            egui::Stroke::new(RING_STROKE_ACCENT, ring_shadow),
        );

        /* WHY: Highlight segment makes the ring look metallic/3D. */
        ui.painter().line_segment(
            [
                ring_center - egui::vec2(HIGHLIGHT_X1, HIGHLIGHT_Y1),
                ring_center - egui::vec2(HIGHLIGHT_X2, HIGHLIGHT_Y2),
            ],
            egui::Stroke::new(
                HIGHLIGHT_STROKE_W,
                WHITE.linear_multiply(HIGHLIGHT_ALPHA_MUL),
            ),
        );

        if response.clicked() {
            let next_mode = if is_code {
                ViewMode::PreviewOnly
            } else {
                ViewMode::CodeOnly
            };
            return Some(crate::app_state::AppAction::SetViewMode(next_mode));
        }

        None
    }
}
