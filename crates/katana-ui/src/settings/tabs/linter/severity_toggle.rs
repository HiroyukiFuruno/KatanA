use crate::i18n::LinterTranslations;
use eframe::egui;
use katana_platform::settings::types::RuleSeverity;

pub(crate) const SEVERITY_TOGGLE_WIDTH: f32 = 180.0;
pub(crate) const SEVERITY_SEGMENT_HEIGHT: f32 = 24.0;
const SEVERITY_RADIUS: f32 = 6.0;

const DIVIDER_PADDING: f32 = 3.0;
const SEVERITY_FONT_SIZE: f32 = 11.0;

pub(crate) struct SeverityToggleOps;

/* WHY: 3-segment control styled like a modern pill/chip toggle.
 * Each segment is a clickable region; the active one is filled with the selection colour.
 * Layout: [  Ignore  |  Warning  |  Error  ]
 * Returns the newly selected severity (unchanged if nothing was clicked). */
impl SeverityToggleOps {
    pub(crate) fn render_severity_segment_toggle(
        ui: &mut egui::Ui,
        id_prefix: &str,
        current: RuleSeverity,
        msgs: &LinterTranslations,
    ) -> RuleSeverity {
        const SEG_COUNT: usize = 3;
        let segments: [(RuleSeverity, &str); SEG_COUNT] = [
            (RuleSeverity::Ignore, &msgs.severity_ignore),
            (RuleSeverity::Warning, &msgs.severity_warning),
            (RuleSeverity::Error, &msgs.severity_error),
        ];

        let total_w = SEVERITY_TOGGLE_WIDTH;
        let h = SEVERITY_SEGMENT_HEIGHT;
        let seg_w = total_w / SEG_COUNT as f32;

        let (outer_rect, _) = ui.allocate_exact_size(egui::vec2(total_w, h), egui::Sense::hover());

        if !ui.is_rect_visible(outer_rect) {
            return current;
        }

        let selection_color = ui.visuals().selection.bg_fill;
        let inactive_bg = ui.visuals().widgets.inactive.bg_fill;
        let border_color = ui.visuals().widgets.inactive.bg_stroke.color;
        let text_color_active = ui.visuals().selection.stroke.color;
        let text_color_inactive = ui.visuals().widgets.inactive.fg_stroke.color;

        /* WHY: Paint the outer rounded container background. */
        ui.painter().rect(
            outer_rect,
            SEVERITY_RADIUS,
            inactive_bg,
            egui::Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );

        let mut new_severity = current;

        for (i, (sev, label)) in segments.iter().enumerate() {
            let seg_rect = egui::Rect::from_min_size(
                egui::pos2(outer_rect.min.x + i as f32 * seg_w, outer_rect.min.y),
                egui::vec2(seg_w, h),
            );

            let seg_id = ui.id().with(format!("{}_{}", id_prefix, i));
            let sense = egui::Sense::click();
            let seg_response = ui.interact(seg_rect, seg_id, sense);

            let is_active = *sev == current;
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
                    nw: SEVERITY_RADIUS as u8,
                    sw: SEVERITY_RADIUS as u8,
                    ne: 0,
                    se: 0,
                }
            } else if i == SEG_COUNT - 1 {
                egui::CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: SEVERITY_RADIUS as u8,
                    se: SEVERITY_RADIUS as u8,
                }
            } else {
                egui::CornerRadius::ZERO
            };

            ui.painter().rect_filled(seg_rect, corner_radius, bg);

            /* WHY: Draw divider lines between segments (not at left edge). */
            if i > 0 {
                let div_x = seg_rect.min.x;
                ui.painter().line_segment(
                    [
                        egui::pos2(div_x, seg_rect.min.y + DIVIDER_PADDING),
                        egui::pos2(div_x, seg_rect.max.y - DIVIDER_PADDING),
                    ],
                    egui::Stroke::new(1.0, border_color),
                );
            }

            let text_col = if is_active {
                text_color_active
            } else {
                text_color_inactive
            };
            let galley = ui.painter().layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(SEVERITY_FONT_SIZE),
                text_col,
            );
            let text_pos = egui::pos2(
                seg_rect.center().x - galley.size().x / 2.0,
                seg_rect.center().y - galley.size().y / 2.0,
            );
            ui.painter().galley(text_pos, galley, text_col);

            if seg_response.clicked() {
                new_severity = *sev;
            }
        }

        new_severity
    }
}
