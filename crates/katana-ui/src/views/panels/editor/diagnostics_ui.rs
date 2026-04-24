use super::types::EditorLogicOps;
use eframe::egui;

pub(crate) struct EditorDiagnostics;

impl EditorDiagnostics {
    pub(crate) fn render_diagnostics(
        ui: &mut egui::Ui,
        buffer: &str,
        galley: &std::sync::Arc<egui::Galley>,
        response_rect: &egui::Rect,
        diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
        action: &mut crate::app_state::AppAction,
    ) {
        for diag in diagnostics {
            if diag.official_meta.is_none() {
                continue;
            }

            let start_line = diag.range.start_line.saturating_sub(1);
            let start_col = diag.range.start_column.saturating_sub(1);
            let end_line = diag.range.end_line.saturating_sub(1);
            let end_col = diag.range.end_column.saturating_sub(1);

            let start_idx = EditorLogicOps::line_col_to_char_index(buffer, start_line, start_col);
            let end_idx = EditorLogicOps::line_col_to_char_index(buffer, end_line, end_col);

            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                let color = match diag.severity {
                    katana_linter::rules::markdown::DiagnosticSeverity::Error => {
                        ui.visuals().error_fg_color
                    }
                    katana_linter::rules::markdown::DiagnosticSeverity::Warning => {
                        ui.visuals().warn_fg_color
                    }
                    katana_linter::rules::markdown::DiagnosticSeverity::Info => {
                        ui.visuals().text_color()
                    }
                };

                let match_start = egui::text::CCursor {
                    index: start,
                    prefer_next_row: false,
                };
                let match_end = egui::text::CCursor {
                    index: end,
                    prefer_next_row: false,
                };

                let start_row = galley.layout_from_cursor(match_start).row;
                let end_row = galley.layout_from_cursor(match_end).row;

                Self::paint_squiggly(
                    ui,
                    galley,
                    response_rect,
                    match_start,
                    match_end,
                    start_row,
                    end_row,
                    color,
                    diag,
                    action,
                    diagnostics,
                );
            }
        }
    }

    /* WHY: allow(nesting_depth) */
    #[allow(clippy::too_many_arguments)]
    fn paint_squiggly(
        ui: &mut egui::Ui,
        galley: &std::sync::Arc<egui::Galley>,
        response_rect: &egui::Rect,
        match_start: egui::text::CCursor,
        match_end: egui::text::CCursor,
        start_row: usize,
        end_row: usize,
        color: egui::Color32,
        diag: &katana_linter::rules::markdown::MarkdownDiagnostic,
        action: &mut crate::app_state::AppAction,
        all_diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
    ) {
        for row_idx in start_row..=end_row {
            let Some(placed_row) = galley.rows.get(row_idx) else {
                continue;
            };
            let row_rect = placed_row.rect();
            let (left_x, right_x) = if row_idx == start_row {
                let pos_start = galley.pos_from_cursor(match_start);
                (
                    pos_start.min.x.max(0.0),
                    if start_row == end_row {
                        pos_start.min.x
                    } else {
                        row_rect.right()
                    },
                )
            } else if row_idx == end_row {
                let pos_end = galley.pos_from_cursor(match_end);
                (row_rect.left().max(0.0), pos_end.max.x.max(0.0))
            } else {
                (row_rect.left().max(0.0), row_rect.right())
            };

            let right_x = if start_row == end_row {
                let pos_end = galley.pos_from_cursor(match_end);
                pos_end.max.x.max(0.0)
            } else {
                right_x
            };

            let min_x = response_rect.min.x + left_x;
            let max_x = response_rect.min.x + right_x;
            /* WHY: Draw squiggly line near the bottom of the row */
            let y_mid = response_rect.min.y + row_rect.bottom() - 1.0;

            if max_x > min_x {
                super::diagnostics_hover::DiagnosticsHoverOps::draw_wave(
                    ui, min_x, max_x, y_mid, color,
                );

                let hover_rect = egui::Rect::from_min_max(
                    egui::pos2(min_x, response_rect.min.y + row_rect.top()),
                    egui::pos2(max_x, response_rect.min.y + row_rect.bottom()),
                );

                let id = ui.id().with((
                    "diag_hover",
                    &diag.rule_id,
                    diag.range.start_line,
                    diag.range.start_column,
                ));
                let resp = ui.interact(hover_rect, id, egui::Sense::hover());

                resp.on_hover_ui(|ui| {
                    if let Some(meta) = diag.official_meta.as_ref() {
                        super::diagnostics_hover::DiagnosticsHoverOps::show_hover_ui(
                            ui,
                            diag,
                            meta,
                            all_diagnostics,
                            action,
                        );
                    }
                });
            }
        }
    }
}
