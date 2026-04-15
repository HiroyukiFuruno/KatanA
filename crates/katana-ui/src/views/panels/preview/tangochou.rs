use crate::app_state::ViewMode;
use crate::theme_bridge::{ThemeBridgeOps, WHITE};
use eframe::egui;

const CARD_HEIGHT: f32 = 42.0;
const CARD_PADDING: f32 = 12.0;
const CARD_BACK_ANGLE_DEG: f32 = 12.0;
const CARD_HOVER_BACK_EXTRA_DEG: f32 = 2.0;
const CARD_HOVER_FRONT_OFFSET_DEG: f32 = 1.5;
const RING_RADIUS_OUTER: f32 = 9.5;
const RING_RADIUS_INNER: f32 = 6.5;
const RING_RADIUS: f32 = 8.0;
const RING_STROKE_MAIN: f32 = 3.0;
const RING_STROKE_ACCENT: f32 = 1.0;
const RING_X_OFFSET: f32 = 16.0;
const HOLE_RADIUS: f32 = 5.0;
const SHADOW_OFFSET: egui::Vec2 = egui::vec2(2.0, 4.0);
const SHADOW_ALPHA: u8 = 45;
const RING_ALPHA: u8 = 60;
const HIGHLIGHT_ALPHA_MUL: f32 = 0.6;
const CARD_CENTER_RATIO: f32 = 0.5;
const TEXT_SIZE: f32 = 15.0;
const TEXT_X_OFFSET: f32 = 8.0;
const CARD_EXTRA_H_PAD: f32 = 12.0;
const CARD_TOP_OFFSET: f32 = 4.0;
const FRONT_BORDER_W: f32 = 1.5;
const BACK_BORDER_W: f32 = 1.0;
const HIGHLIGHT_Y1: f32 = 6.0;
const HIGHLIGHT_X1: f32 = 4.0;
const HIGHLIGHT_Y2: f32 = 8.0;
const HIGHLIGHT_X2: f32 = 1.0;
const HIGHLIGHT_STROKE_W: f32 = 1.5;
const CARD_GRAY_DARK_BACK: u8 = 35;
const CARD_GRAY_LIGHT_BACK: u8 = 225;
const CARD_GRAY_DARK_FRONT: u8 = 65;
const CARD_GRAY_LIGHT_FRONT: u8 = 255;
const RING_GRAY_DARK: u8 = 120;
const RING_GRAY_LIGHT: u8 = 180;

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
        let back_bg = if is_dark {
            ThemeBridgeOps::from_gray(CARD_GRAY_DARK_BACK)
        } else {
            ThemeBridgeOps::from_gray(CARD_GRAY_LIGHT_BACK)
        };
        let front_bg = if is_dark {
            ThemeBridgeOps::from_gray(CARD_GRAY_DARK_FRONT)
        } else {
            ThemeBridgeOps::from_gray(CARD_GRAY_LIGHT_FRONT)
        };
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
                rotate_pos(rect.left_top()),
                rotate_pos(rect.right_top()),
                rotate_pos(rect.right_bottom()),
                rotate_pos(rect.left_bottom()),
            ];

            if is_front {
                let shadow_corners = corners.map(|p| p + SHADOW_OFFSET);
                ui.painter().add(egui::Shape::convex_polygon(
                    shadow_corners.into(),
                    ThemeBridgeOps::from_black_alpha(SHADOW_ALPHA),
                    egui::Stroke::NONE,
                ));
            }

            let border_w = if is_front { FRONT_BORDER_W } else { BACK_BORDER_W };
            let border_col = if is_front { selection_col } else { stroke_col };
            ui.painter()
                .add(egui::Shape::convex_polygon(corners.into(), bg, egui::Stroke::new(border_w, border_col)));

            let hole_pos = rotate_pos(ring_center);
            ui.painter()
                .circle_filled(hole_pos, HOLE_RADIUS, ui.visuals().window_fill());
            ui.painter()
                .circle_stroke(hole_pos, HOLE_RADIUS, egui::Stroke::new(BACK_BORDER_W, stroke_col));

            let text_color = if is_front {
                ui.visuals().text_color()
            } else {
                ui.visuals().text_color().linear_multiply(HIGHLIGHT_ALPHA_MUL)
            };
            let galley = ui
                .painter()
                .layout_no_wrap(text.to_owned(), egui::FontId::proportional(TEXT_SIZE), text_color);

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
        let ring_color = if is_dark {
            ThemeBridgeOps::from_gray(RING_GRAY_DARK)
        } else {
            ThemeBridgeOps::from_gray(RING_GRAY_LIGHT)
        };
        let ring_shadow = ThemeBridgeOps::from_black_alpha(RING_ALPHA);
        ui.painter()
            .circle_stroke(ring_center, RING_RADIUS, egui::Stroke::new(RING_STROKE_MAIN, ring_color));
        ui.painter()
            .circle_stroke(ring_center, RING_RADIUS_OUTER, egui::Stroke::new(RING_STROKE_ACCENT, ring_shadow));
        ui.painter()
            .circle_stroke(ring_center, RING_RADIUS_INNER, egui::Stroke::new(RING_STROKE_ACCENT, ring_shadow));

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
