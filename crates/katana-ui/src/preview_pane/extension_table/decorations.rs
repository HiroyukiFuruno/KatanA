use egui::{Rect, Shape, Ui, layers::ShapeIdx, pos2};

/* WHY: Match pulldown.rs reference line 1857-1858.
 * Frame::group outer_margin is symmetric(5, 0), so response.rect does NOT include outer_margin.
 * To make backgrounds span to the visual border edges, expand by 10px (outer_margin * 2). */
const BG_EXPANSION: f32 = 10.0;
const TABLE_HEADER_ALPHA: f32 = 0.3; /* WHY: Aesthetic choice for header background transparency. */

pub(crate) struct KatanaTableDecorations;

impl KatanaTableDecorations {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_decorations(
        ui: &mut Ui,
        frame_rect: Rect,
        header_bg_idx: Option<ShapeIdx>,
        header_bottom_y: Option<f32>,
        col_boundaries: &[f32],
        num_rows: usize,
        row_bg_indices: &[(usize, ShapeIdx)],
        row_bounds: &[(f32, f32)],
    ) {
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;

        /* WHY: Match pulldown.rs reference line 1857-1858.
         * bg_left/bg_right expand beyond frame_rect to cover the outer_margin area,
         * ensuring backgrounds snap exactly to the outer stroke border. */
        let bg_left = frame_rect.left() - BG_EXPANSION;
        let bg_right = frame_rect.right() + BG_EXPANSION;
        let bg_top = frame_rect.top();
        let bg_bottom = frame_rect.bottom();
        let bg_x_range = bg_left..=bg_right;

        /* WHY: Draw Header Background */
        if let (Some(shape_idx), Some(mut h_bottom)) = (header_bg_idx, header_bottom_y) {
            let header_bg_color = ui
                .visuals()
                .selection
                .bg_fill
                .gamma_multiply(TABLE_HEADER_ALPHA);

            let corner_radius = ui.visuals().widgets.noninteractive.corner_radius;
            let header_rounding = egui::CornerRadius {
                nw: corner_radius.nw,
                ne: corner_radius.ne,
                sw: if num_rows == 0 { corner_radius.sw } else { 0 },
                se: if num_rows == 0 { corner_radius.se } else { 0 },
            };

            /* WHY: header-only table → fill the entire frame (pulldown.rs line 1865-1867). */
            if num_rows == 0 {
                h_bottom = bg_bottom;
            }

            ui.painter().set(
                shape_idx,
                Shape::rect_filled(
                    Rect::from_min_max(pos2(bg_left, bg_top), pos2(bg_right, h_bottom)),
                    header_rounding,
                    header_bg_color,
                ),
            );

            if num_rows > 0 {
                ui.painter().hline(bg_x_range.clone(), h_bottom, stroke);
            }
        }

        /* WHY: Draw Row backgrounds (Zebra striping) — pulldown.rs reference line 1898-1934. */
        for &(row_idx, shape_idx) in row_bg_indices {
            if let Some(&(top_y, mut bottom_y)) = row_bounds.get(row_idx) {
                let corner_radius = ui.visuals().widgets.noninteractive.corner_radius;
                let is_last = row_idx == num_rows.saturating_sub(1);

                /* WHY: Extend last row to fill the bottom margin gap (pulldown.rs line 1901-1903). */
                if is_last {
                    bottom_y = bg_bottom;
                }

                let row_rounding = egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: if is_last { corner_radius.sw } else { 0 },
                    se: if is_last { corner_radius.se } else { 0 },
                };

                ui.painter().set(
                    shape_idx,
                    Shape::rect_filled(
                        Rect::from_min_max(pos2(bg_left, top_y), pos2(bg_right, bottom_y)),
                        row_rounding,
                        ui.visuals().faint_bg_color,
                    ),
                );
            }
        }

        /* WHY: Draw vertical column dividers (pulldown.rs line 1939-1942). */
        for &x in col_boundaries.iter() {
            ui.painter().vline(x, bg_top..=bg_bottom, stroke);
        }

        /* WHY: Draw horizontal separator lines between body rows (pulldown.rs line 1951-1955). */
        for (_, bottom_y) in row_bounds.iter().take(num_rows.saturating_sub(1)) {
            ui.painter().hline(bg_x_range.clone(), *bottom_y, stroke);
        }

        /* WHY: Draw outer frame border on top (pulldown.rs line 1957-1968).
         * Use bg_left/bg_right to match the expanded background area. */
        let border_rect = Rect::from_min_max(pos2(bg_left, bg_top), pos2(bg_right, bg_bottom));
        ui.painter().add(egui::Shape::rect_stroke(
            border_rect,
            ui.visuals().widgets.noninteractive.corner_radius,
            stroke,
            egui::StrokeKind::Inside,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, pos2, vec2};

    #[test]
    fn test_draw_decorations_basic() {
        let ctx = Context::default();
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 200.0));
        let builder = egui::UiBuilder::new().max_rect(rect);
        let mut ui = Ui::new(ctx, egui::Id::new("test"), builder);

        let frame_rect = Rect::from_min_max(pos2(10.0, 10.0), pos2(110.0, 110.0));
        let boundaries = vec![30.0, 80.0];

        KatanaTableDecorations::draw_decorations(
            &mut ui,
            frame_rect,
            None,
            Some(30.0),
            &boundaries,
            0,
            &[],
            &[],
        );
    }
}
