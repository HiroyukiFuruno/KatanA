use std::ops::Range;

use egui_commonmark::TaskListAction;

const STROKE_WIDTH_MIN: f32 = 1.5;
const GAP_WIDTH: f32 = 8.0;
const DRAW_ZERO: f32 = 0.0;
const CHECK_PT1_X: f32 = -0.25;
const CHECK_PT2_X: f32 = -0.05;
const CHECK_PT2_Y: f32 = 0.25;
const CHECK_PT3_X: f32 = 0.3;
const CHECK_PT3_Y: f32 = -0.25;
const PROG_HALF_W: f32 = 0.35;

const HIGHLIGHT_ROUNDING: f32 = 1.0;

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
            if state == '/' {
                let half_w = width * PROG_HALF_W;
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-half_w, half_w),
                        center + egui::vec2(half_w, -half_w),
                    ],
                    stroke,
                );
            } else {
                let half_w = width * PROG_HALF_W;
                ui.painter().line_segment(
                    [
                        center - egui::vec2(half_w, DRAW_ZERO),
                        center + egui::vec2(half_w, DRAW_ZERO),
                    ],
                    stroke,
                );
            }
        }
    }

    if mutable {
        if response.clicked() {
            let new_state = match state {
                ' ' => 'x',
                '/' | '-' | '~' => 'x',
                'x' | 'X' => ' ',
                _ => ' ',
            };
            events.push(TaskListAction {
                span: span.clone(),
                new_state,
            });
        }

        response.context_menu(|ui| {
            if ui
                .button(crate::i18n::get().markdown.task_todo.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: ' ',
                });
                ui.close();
            }
            if ui
                .button(crate::i18n::get().markdown.task_in_progress.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: '/',
                });
                ui.close();
            }
            if ui
                .button(crate::i18n::get().markdown.task_done.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: 'x',
                });
                ui.close();
            }
        });
    }

    // Add interactable margin between checkbox and text
    let mut gap_response = ui.allocate_response(
        egui::vec2(GAP_WIDTH, interact_size.y.max(icon_width)), // Match height of checkbox interaction
        if mutable {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    if mutable {
        gap_response = gap_response.on_hover_cursor(egui::CursorIcon::PointingHand);
        if gap_response.clicked() {
            let new_state = match state {
                ' ' => 'x',
                '/' | '-' | '~' => 'x',
                'x' | 'X' => ' ',
                _ => ' ',
            };
            events.push(TaskListAction {
                span: span.clone(),
                new_state,
            });
        }
        gap_response.context_menu(|ui| {
            if ui
                .button(crate::i18n::get().markdown.task_todo.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: ' ',
                });
                ui.close();
            }
            if ui
                .button(crate::i18n::get().markdown.task_in_progress.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: '/',
                });
                ui.close();
            }
            if ui
                .button(crate::i18n::get().markdown.task_done.clone())
                .clicked()
            {
                events.push(TaskListAction {
                    span: span.clone(),
                    new_state: 'x',
                });
                ui.close();
            }
        });
    }
}

pub fn katana_task_context_menu(
    response: &egui::Response,
    state: char,
    span: Range<usize>,
    _mutable: bool,
    events: &mut Vec<TaskListAction>,
) {
    if response.clicked() {
        let new_state = match state {
            ' ' => 'x',
            '/' | '-' | '~' => 'x',
            'x' | 'X' => ' ',
            _ => ' ',
        };
        events.push(TaskListAction {
            span: span.clone(),
            new_state,
        });
    }

    response.context_menu(|ui| {
        if ui
            .button(crate::i18n::get().markdown.task_todo.clone())
            .clicked()
        {
            events.push(TaskListAction {
                span: span.clone(),
                new_state: ' ',
            });
            ui.close();
        }
        if ui
            .button(crate::i18n::get().markdown.task_in_progress.clone())
            .clicked()
        {
            events.push(TaskListAction {
                span: span.clone(),
                new_state: '/',
            });
            ui.close();
        }
        if ui
            .button(crate::i18n::get().markdown.task_done.clone())
            .clicked()
        {
            events.push(TaskListAction {
                span: span.clone(),
                new_state: 'x',
            });
            ui.close();
        }
    });
}

/// Factory for the list-item highlight/hover callback.
///
/// All highlight logic is owned by katana-ui; pulldown.rs only provides
/// the correct bounding `Rect` (from `horizontal_wrapped` response) and
/// the item's source `Range<usize>`.
///
/// Returns `(active_highlighted, hovered)`.
pub fn katana_list_item_highlight(
    active_char_range: Option<Range<usize>>,
    active_bg_color: egui::Color32,
    hover_bg_color: egui::Color32,
) -> impl Fn(&mut egui::Ui, egui::Rect, &Range<usize>) -> (bool, bool) {
    move |ui: &mut egui::Ui, rect: egui::Rect, span: &Range<usize>| {
        let mut highlighted = false;
        let mut hovered = false;

        // Active highlight (editor cursor line)
        if let Some(ref active) = active_char_range
            && active.start <= span.end
            && active.end >= span.start
        {
            highlighted = true;
            ui.painter()
                .rect_filled(rect, HIGHLIGHT_ROUNDING, active_bg_color);
        }

        // Hover highlight (mouse pointer).
        if let Some(pos) = ui.ctx().pointer_hover_pos()
            && rect.contains(pos)
        {
            hovered = true;
            ui.painter()
                .rect_filled(rect, HIGHLIGHT_ROUNDING, hover_bg_color);
        }

        (highlighted, hovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_and_inspect(md: &str, hover_pos: egui::Pos2) -> Vec<Range<usize>> {
        let ctx = egui::Context::default();
        let mut hovered_spans = Vec::new();

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        ));
        raw_input.events.push(egui::Event::PointerMoved(hover_pos));

        let _ = ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut cache = egui_commonmark::CommonMarkCache::default();

                let highlight_fn = super::katana_list_item_highlight(
                    None,
                    crate::theme_bridge::TRANSPARENT,
                    crate::theme_bridge::TRANSPARENT,
                );

                let viewer = egui_commonmark::CommonMarkViewer::new()
                    .hovered_spans(&mut hovered_spans)
                    .custom_list_item_highlight_fn(Some(&highlight_fn));

                viewer.show(ui, &mut cache, md);
            });
        });

        hovered_spans
    }

    #[test]
    fn hover_on_task_list_item_produces_single_span() {
        let md = "- [x] **bold** task text\n- [ ] second item\n";
        // Center of first list item area
        let spans = render_and_inspect(md, egui::pos2(50.0, 15.0));
        assert!(
            spans.len() <= 1,
            "Expected at most 1 hovered span for task list item, got {}: {:?}",
            spans.len(),
            spans
        );
    }

    #[test]
    fn hover_on_mixed_content_produces_single_span() {
        let md = "## Heading\n\n- item one\n- item two\n\nParagraph after list.\n";
        // Target the list item area (below heading)
        let spans = render_and_inspect(md, egui::pos2(50.0, 65.0));
        assert!(
            spans.len() <= 1,
            "Expected at most 1 hovered span, got {}: {:?}",
            spans.len(),
            spans
        );
    }

    #[test]
    fn hover_at_item_boundary_produces_single_span() {
        // Adjacent list items share boundary (item1.max_y == item2.min_y).
        // Hovering at that boundary must NOT produce 2 hovered spans.
        let md = "- item one\n- item two\n";
        // Item height is typically 15px starting at y=8.
        // Boundary between items is at y=23 (8+15).
        let boundary_y = 23.0;
        let spans = render_and_inspect(md, egui::pos2(50.0, boundary_y));
        assert!(
            spans.len() <= 1,
            "At boundary y={}, expected at most 1 hovered span, got {}: {:?}",
            boundary_y,
            spans.len(),
            spans
        );
    }
}
