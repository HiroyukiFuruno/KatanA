use super::types::*;
use egui_commonmark::TaskListAction;
use std::ops::Range;

impl MarkdownHooksOps {
    pub fn katana_task_box(
        ui: &mut egui::Ui,
        state: char,
        span: Range<usize>,
        mutable: bool,
        events: &mut Vec<TaskListAction>,
    ) {
        let is_checked = state == 'x' || state == 'X';
        let is_progress = state == '/' || state == '-' || state == '~';
        let is_active = is_checked || is_progress;

        let icon_width = ui.spacing().icon_width;
        let interact_size = ui.spacing().interact_size;
        let desired_size = egui::vec2(icon_width, interact_size.y.max(icon_width));

        let (rect, response) = ui.allocate_exact_size(
            desired_size,
            if mutable {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&response, is_active);
            let rounding = ui.visuals().widgets.noninteractive.corner_radius;

            let (_, big_icon_rect) = ui.spacing().icon_rectangles(rect);

            ui.painter().rect(
                big_icon_rect.expand(visuals.expansion),
                rounding,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            let stroke_width = ui
                .visuals()
                .widgets
                .noninteractive
                .fg_stroke
                .width
                .max(STROKE_WIDTH_MIN);
            let stroke = egui::Stroke::new(stroke_width, visuals.fg_stroke.color);
            let center = big_icon_rect.center();
            let width = big_icon_rect.width();

            if is_checked {
                ui.painter().line_segment(
                    [
                        center + egui::vec2(width * CHECK_PT1_X, DRAW_ZERO),
                        center + egui::vec2(width * CHECK_PT2_X, width * CHECK_PT2_Y),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        center + egui::vec2(width * CHECK_PT2_X, width * CHECK_PT2_Y),
                        center + egui::vec2(width * CHECK_PT3_X, width * CHECK_PT3_Y),
                    ],
                    stroke,
                );
            } else if is_progress {
                draw_progress_indicator(ui, state, center, width, stroke);
            }
        }

        if mutable {
            if response.clicked() {
                let new_state = toggle_task_state(state);
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state,
                });
            }
            MarkdownHooksOps::push_context_menu(&response, &span, events);
        }

        /* WHY: Add interactable margin between checkbox and text */
        let sense = if mutable {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let mut gap_response = ui.allocate_response(
            egui::vec2(GAP_WIDTH, interact_size.y.max(icon_width)),
            sense,
        );

        if mutable {
            gap_response = gap_response.on_hover_cursor(egui::CursorIcon::PointingHand);
            if gap_response.clicked() {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: toggle_task_state(state),
                });
            }
            MarkdownHooksOps::push_context_menu(&gap_response, &span, events);
        }
    }
}

fn draw_progress_indicator(
    ui: &mut egui::Ui,
    state: char,
    center: egui::Pos2,
    width: f32,
    stroke: egui::Stroke,
) {
    let half_w = width * PROG_HALF_W;
    if state == '/' {
        ui.painter().line_segment(
            [
                center + egui::vec2(-half_w, half_w),
                center + egui::vec2(half_w, -half_w),
            ],
            stroke,
        );
    } else {
        ui.painter().line_segment(
            [
                center - egui::vec2(half_w, DRAW_ZERO),
                center + egui::vec2(half_w, DRAW_ZERO),
            ],
            stroke,
        );
    }
}

pub(super) fn toggle_task_state(state: char) -> char {
    match state {
        ' ' => 'x',
        '/' | '-' | '~' => 'x',
        'x' | 'X' => ' ',
        _ => ' ',
    }
}
