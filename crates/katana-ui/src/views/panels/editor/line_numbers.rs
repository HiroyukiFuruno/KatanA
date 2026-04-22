use eframe::egui;

pub(crate) struct EditorLineNumbers;

pub(crate) struct LineNumberParams<'a> {
    pub galley: &'a std::sync::Arc<egui::Galley>,
    pub response_rect: &'a egui::Rect,
    pub ln_rect: &'a egui::Rect,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub current_cursor_y: Option<f32>,
    pub ln_text: Option<egui::Color32>,
    pub ln_active_text: Option<egui::Color32>,
    pub left_margin: f32,
    pub line_number_pad_right: f32,
    pub diagnostics: &'a [katana_linter::rules::markdown::MarkdownDiagnostic],
    pub action: &'a mut crate::app_state::AppAction,
}

struct RowRenderParams {
    p: usize,
    top_y: f32,
    y: f32,
    row_height: f32,
}

impl EditorLineNumbers {
    pub(crate) fn render(ui: &mut egui::Ui, params: LineNumberParams<'_>) {
        let LineNumberParams {
            galley,
            response_rect,
            ln_rect,
            scroll,
            current_cursor_y,
            ln_text,
            ln_active_text,
            left_margin,
            line_number_pad_right,
            diagnostics,
            action,
        } = params;
        let clip_rect = ui.clip_rect().expand(100.0);
        let mut p = 0;
        let mut is_start_of_para = true;

        for row in &galley.rows {
            let row_rect = row.rect();
            let top_y = row_rect.min.y;
            let y = response_rect.min.y + top_y;
            let is_visible = is_start_of_para
                && y <= clip_rect.max.y
                && (y + row_rect.height()) >= clip_rect.min.y;

            if is_visible {
                let rp = RowRenderParams {
                    p,
                    top_y,
                    y,
                    row_height: row_rect.height(),
                };
                super::row_diagnostics::RowDiagnosticsRenderer::render(
                    ui,
                    diagnostics,
                    p,
                    y,
                    ln_rect,
                    row_rect.height(),
                    &mut *action,
                );
                Self::render_row_number(
                    ui,
                    rp,
                    ln_rect,
                    scroll,
                    current_cursor_y,
                    ln_text,
                    ln_active_text,
                    left_margin,
                    line_number_pad_right,
                    diagnostics,
                );
            }

            is_start_of_para = row.ends_with_newline;
            if row.ends_with_newline {
                p += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_row_number(
        ui: &mut egui::Ui,
        rp: RowRenderParams,
        ln_rect: &egui::Rect,
        scroll: &mut crate::app_state::ScrollState,
        current_cursor_y: Option<f32>,
        ln_text: Option<egui::Color32>,
        ln_active_text: Option<egui::Color32>,
        left_margin: f32,
        line_number_pad_right: f32,
        _diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
    ) {
        let RowRenderParams {
            p,
            top_y,
            y,
            row_height,
        } = rp;
        let is_current = current_cursor_y == Some(top_y);
        let text = format!("{}", p + 1);
        let color: egui::Color32 = if is_current {
            ln_active_text.unwrap_or_else(|| -> egui::Color32 { ui.visuals().text_color() })
        } else {
            const LINE_NUMBER_INACTIVE_ALPHA: f32 = 0.3;
            ln_text.unwrap_or_else(|| -> egui::Color32 {
                ui.visuals()
                    .text_color()
                    .linear_multiply(LINE_NUMBER_INACTIVE_ALPHA)
            })
        };
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        let label_rect = egui::Rect::from_min_size(
            egui::pos2(ln_rect.min.x, y),
            egui::vec2(left_margin - line_number_pad_right, row_height),
        );
        let mut text_rt = egui::RichText::new(text).color(color).font(font_id);
        if is_current {
            text_rt = text_rt.strong();
        }
        let label_for_measuring = egui::Label::new(text_rt.clone()).selectable(false);
        let galley_ln = label_for_measuring.layout_in_ui(ui);
        let offset_x = (label_rect.width() - galley_ln.1.rect.width()).max(0.0);
        let tight_rect = egui::Rect::from_min_size(
            label_rect.min + egui::vec2(offset_x, 0.0),
            galley_ln.1.rect.size(),
        );
        let resp = ui.interact(label_rect, ui.id().with(p), egui::Sense::click());
        if resp.clicked() {
            scroll.scroll_to_line = Some(p);
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        ui.put(tight_rect, egui::Label::new(text_rt).selectable(false));
    }
}
