use egui::{Rect, Shape, Ui, layers::ShapeIdx, pos2};

const TABLE_HEADER_ALPHA: f32 = 0.3;

pub(crate) struct KatanaTableDecorations;

impl KatanaTableDecorations {
    fn header_fill_rect(frame_rect: Rect, header_bounds: (f32, f32), num_rows: usize) -> Rect {
        let (_, header_bottom) = header_bounds;
        let bottom = if num_rows == 0 {
            frame_rect.bottom()
        } else {
            header_bottom.max(frame_rect.top()).min(frame_rect.bottom())
        };
        Rect::from_min_max(frame_rect.min, pos2(frame_rect.right(), bottom))
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
        let sw = stroke.width;
        /* WHY: Inset fills by stroke.width so they sit entirely inside the rect_stroke border. */
        let fill_left = frame_rect.left() + sw;
        let fill_right = frame_rect.right() - sw;
        let fill_top = frame_rect.top() + sw;
        let fill_bottom = frame_rect.bottom() - sw;
        let x_range = frame_rect.left()..=frame_rect.right();
        let y_range = frame_rect.top()..=frame_rect.bottom();

        if let (Some(shape_idx), Some(bounds)) = (header_bg_idx, header_bounds) {
            let fill = Self::header_fill_rect(frame_rect, bounds, num_rows);
            /* WHY: Clamp header fill to inset bounds so it sits inside the border stroke. */
            let fill_clamped = Rect::from_min_max(
                pos2(fill_left, fill_top),
                pos2(fill_right, fill.bottom().min(fill_bottom)),
            );
            let color = ui
                .visuals()
                .selection
                .bg_fill
                .gamma_multiply(TABLE_HEADER_ALPHA);
            ui.painter().set(
                shape_idx,
                Shape::rect_filled(fill_clamped, egui::CornerRadius::ZERO, color),
            );
            /* WHY: Draw header/body separator only when body rows exist. */
            if num_rows > 0 {
                let (_, header_bottom) = bounds;
                ui.painter().hline(x_range.clone(), header_bottom, stroke);
            }
        }

        for &(row_idx, shape_idx) in row_bg_indices {
            if let Some(&(top_y, mut bottom_y)) = row_bounds.get(row_idx) {
                if row_idx == num_rows.saturating_sub(1) {
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

        for &x in col_boundaries {
            ui.painter().vline(x, y_range.clone(), stroke);
        }

        for (_, bottom_y) in row_bounds.iter().take(num_rows.saturating_sub(1)) {
            ui.painter().hline(x_range.clone(), *bottom_y, stroke);
        }

        ui.painter().add(egui::Shape::rect_stroke(
            frame_rect,
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
