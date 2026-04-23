use eframe::egui;

pub struct SegmentedStringToggle<'a> {
    id_source: egui::Id,
    choices: &'a [&'a str],
    selected: &'a mut String,
    segment_width: f32,
    segment_height: f32,
}

impl<'a> SegmentedStringToggle<'a> {
    pub fn new(
        id_source: impl std::hash::Hash,
        choices: &'a [&'a str],
        selected: &'a mut String,
    ) -> Self {
        const DEFAULT_SEGMENT_WIDTH: f32 = 80.0;
        const DEFAULT_SEGMENT_HEIGHT: f32 = 24.0;
        Self {
            id_source: egui::Id::new(id_source),
            choices,
            selected,
            segment_width: DEFAULT_SEGMENT_WIDTH,
            segment_height: DEFAULT_SEGMENT_HEIGHT,
        }
    }

    pub fn segment_width(mut self, width: f32) -> Self {
        self.segment_width = width;
        self
    }
}

impl<'a> egui::Widget for SegmentedStringToggle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let seg_count = self.choices.len();
        if seg_count == 0 {
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        }

        let total_w = self.segment_width * seg_count as f32;
        let h = self.segment_height;
        let seg_w = self.segment_width;

        let (outer_rect, mut outer_response) =
            ui.allocate_exact_size(egui::vec2(total_w, h), egui::Sense::hover());

        if !ui.is_rect_visible(outer_rect) {
            return outer_response;
        }

        let selection_color = ui.visuals().selection.bg_fill;
        let inactive_bg = ui.visuals().widgets.inactive.bg_fill;
        let mut border_color = ui.visuals().widgets.inactive.bg_stroke.color;

        const DIM_ALPHA_THRESHOLD: u8 = 30;
        const HOVER_BORDER_DIM_FACTOR: f32 = 0.5;

        if border_color.a() < DIM_ALPHA_THRESHOLD {
            border_color = ui
                .visuals()
                .widgets
                .hovered
                .bg_stroke
                .color
                .linear_multiply(HOVER_BORDER_DIM_FACTOR);
        }
        let text_color_active = ui.visuals().selection.stroke.color;
        let text_color_inactive = ui.visuals().widgets.inactive.fg_stroke.color;

        const BORDER_RADIUS: f32 = 6.0;

        /* WHY: Paint the outer rounded container background. */
        ui.painter().rect(
            outer_rect,
            BORDER_RADIUS,
            inactive_bg,
            egui::Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        let mut changed = false;

        for (i, &choice) in self.choices.iter().enumerate() {
            let seg_rect = egui::Rect::from_min_size(
                egui::pos2(outer_rect.min.x + i as f32 * seg_w, outer_rect.min.y),
                egui::vec2(seg_w, h),
            );

            let seg_id = self.id_source.with(i);
            let seg_response = ui.interact(seg_rect, seg_id, egui::Sense::click());

            let is_active = *self.selected == choice;
            let is_hovered = seg_response.hovered();

            /* WHY: Determine background for this segment. */
            let bg = if is_active {
                selection_color
            } else if is_hovered {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                crate::theme_bridge::TRANSPARENT
            };

            /* WHY: Clip painted segment to outer rounded rect to avoid bleeding at corners. */
            let corner_radius = if i == 0 {
                egui::CornerRadius {
                    nw: BORDER_RADIUS as u8,
                    sw: BORDER_RADIUS as u8,
                    ne: 0,
                    se: 0,
                }
            } else if i == seg_count - 1 {
                egui::CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: BORDER_RADIUS as u8,
                    se: BORDER_RADIUS as u8,
                }
            } else {
                egui::CornerRadius::ZERO
            };

            ui.painter().rect_filled(seg_rect, corner_radius, bg);

            /* WHY: Draw divider lines between segments (not at left edge). */
            if i > 0 {
                const DIVIDER_Y_PADDING: f32 = 3.0;
                let div_x = seg_rect.min.x;
                ui.painter().line_segment(
                    [
                        egui::pos2(div_x, seg_rect.min.y + DIVIDER_Y_PADDING),
                        egui::pos2(div_x, seg_rect.max.y - DIVIDER_Y_PADDING),
                    ],
                    egui::Stroke::new(1.0, border_color),
                );
            }

            let text_col = if is_active {
                text_color_active
            } else {
                text_color_inactive
            };
            const FONT_SIZE: f32 = 11.0;
            let galley = ui.painter().layout_no_wrap(
                choice.to_string(),
                egui::FontId::proportional(FONT_SIZE),
                text_col,
            );
            let text_pos = egui::pos2(
                seg_rect.center().x - galley.size().x / 2.0,
                seg_rect.center().y - galley.size().y / 2.0,
            );
            ui.painter().galley(text_pos, galley, text_col);

            if seg_response.clicked() && !is_active {
                *self.selected = choice.to_string();
                changed = true;
            }
        }

        if changed {
            outer_response.mark_changed();
        }

        outer_response
    }
}
