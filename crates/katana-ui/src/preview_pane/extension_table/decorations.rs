use egui::{Rect, Shape, Ui, layers::ShapeIdx, pos2};

const TABLE_HEADER_ALPHA: f32 = 0.3; /* WHY: Aesthetic choice for header background transparency. */
const TABLE_HORIZONTAL_BLEED: f32 = 2.5;
/* WHY: Lift the visual frame slightly so header text has explicit top breathing room.
 * This must stay paired with renderer top spacing to avoid collapsing heading->table margin. */
const TABLE_TOP_BLEED: f32 = 4.0;
/* WHY: The frame response includes trailing row spacing. We inset bottom so
 * the outer border ends at the visual table end, keeping the following block
 * margin intact and avoiding border-overlap with the next heading. */
const TABLE_BOTTOM_INSET: f32 = 5.0;

pub(crate) struct KatanaTableDecorations;

#[derive(Clone, Copy)]
struct TableGeometry {
    border_rect: Rect,
    header_rect: Option<Rect>,
}

impl KatanaTableDecorations {
    fn horizontal_bounds(frame_rect: Rect) -> (f32, f32) {
        (
            frame_rect.left() - TABLE_HORIZONTAL_BLEED,
            frame_rect.right() + TABLE_HORIZONTAL_BLEED,
        )
    }

    fn border_rect(frame_rect: Rect) -> Rect {
        let (left, right) = Self::horizontal_bounds(frame_rect);
        let top = frame_rect.top() - TABLE_TOP_BLEED;
        let bottom = (frame_rect.bottom() - TABLE_BOTTOM_INSET).max(top);
        Rect::from_min_max(pos2(left, top), pos2(right, bottom))
    }

    fn header_rect(border_rect: Rect, header_bounds: (f32, f32), num_rows: usize) -> Rect {
        let (_, header_bottom) = header_bounds;
        /* WHY: Use border_rect.top() so the header fill aligns exactly with the outer
         * rect_stroke border — prevents a visual gap that creates a double-line effect. */
        let top = border_rect.top();
        let bottom = if num_rows == 0 {
            border_rect.bottom()
        } else {
            header_bottom.max(top).min(border_rect.bottom())
        };
        Rect::from_min_max(
            pos2(border_rect.left(), top),
            pos2(border_rect.right(), bottom),
        )
    }

    fn geometry(
        frame_rect: Rect,
        header_bounds: Option<(f32, f32)>,
        num_rows: usize,
    ) -> TableGeometry {
        let border_rect = Self::border_rect(frame_rect);
        let header_rect =
            header_bounds.map(|bounds| Self::header_rect(border_rect, bounds, num_rows));
        TableGeometry {
            border_rect,
            header_rect,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_decorations(
        ui: &mut Ui,
        frame_rect: Rect,
        header_bg_idx: Option<ShapeIdx>,
        header_bounds: Option<(f32, f32)>,
        col_boundaries: &[f32],
        num_rows: usize,
        row_bg_indices: &[(usize, ShapeIdx)],
        row_bounds: &[(f32, f32)],
    ) {
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        let geometry = Self::geometry(frame_rect, header_bounds, num_rows);
        let bg_left = geometry.border_rect.left();
        let bg_right = geometry.border_rect.right();
        let bg_top = geometry.border_rect.top();
        let bg_bottom = geometry.border_rect.bottom();
        let bg_x_range = bg_left..=bg_right;

        /* WHY: Inset fill boundaries by stroke.width so all fills sit entirely
         * INSIDE the outer rect_stroke border. This eliminates the "double line"
         * artifact caused by fill edges overlapping with the stroke band's
         * two anti-aliased edges (outer and inner). */
        let sw = stroke.width;
        let fill_left = bg_left + sw;
        let fill_right = bg_right - sw;
        let fill_top = bg_top + sw;
        let fill_bottom = bg_bottom - sw;

        /* WHY: Draw Header Background */
        if let (Some(shape_idx), Some(header_rect)) = (header_bg_idx, geometry.header_rect) {
            let header_bg_color = ui
                .visuals()
                .selection
                .bg_fill
                .gamma_multiply(TABLE_HEADER_ALPHA);

            /* WHY: Use zero rounding for the fill — it sits inside the border's
             * rounded corners, so fill rounding would create visible gaps. */
            let header_fill_rect = Rect::from_min_max(
                pos2(fill_left, fill_top),
                pos2(fill_right, header_rect.bottom()),
            );

            ui.painter().set(
                shape_idx,
                Shape::rect_filled(header_fill_rect, egui::CornerRadius::ZERO, header_bg_color),
            );

            /* WHY: Draw separator between header and body rows.
             * Uses the logical header_rect.bottom() for position calculation. */
            if num_rows > 0 {
                ui.painter()
                    .hline(bg_x_range.clone(), header_rect.bottom(), stroke);
            }
        }

        /* WHY: Draw Row backgrounds (Zebra striping) — inset from border edges. */
        for &(row_idx, shape_idx) in row_bg_indices {
            if let Some(&(top_y, mut bottom_y)) = row_bounds.get(row_idx) {
                let is_last = row_idx == num_rows.saturating_sub(1);

                /* WHY: Last row fills to the inset bottom boundary. */
                if is_last {
                    bottom_y = fill_bottom;
                }

                ui.painter().set(
                    shape_idx,
                    Shape::rect_filled(
                        Rect::from_min_max(pos2(fill_left, top_y), pos2(fill_right, bottom_y)),
                        egui::CornerRadius::ZERO,
                        ui.visuals().faint_bg_color,
                    ),
                );
            }
        }

        /* WHY: Draw vertical column dividers. */
        for &x in col_boundaries.iter() {
            ui.painter().vline(x, bg_top..=bg_bottom, stroke);
        }

        /* WHY: Draw horizontal separator lines between body rows. */
        for (_, bottom_y) in row_bounds.iter().take(num_rows.saturating_sub(1)) {
            ui.painter().hline(bg_x_range.clone(), *bottom_y, stroke);
        }

        /* WHY: Draw outer frame border with rounded corners.
         * StrokeKind::Inside ensures the stroke occupies [edge, edge + stroke.width].
         * All fills are inset by stroke.width, so they sit adjacent to (but not
         * overlapping) the stroke — cleanly eliminating the double-line artifact. */
        ui.painter().add(egui::Shape::rect_stroke(
            geometry.border_rect,
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
            Some((10.0, 30.0)),
            &boundaries,
            0,
            &[],
            &[],
        );
    }
}
