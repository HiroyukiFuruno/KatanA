use eframe::egui;

pub(crate) struct EditorLineNumbers;

impl EditorLineNumbers {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        galley: &std::sync::Arc<egui::Galley>,
        response_rect: &egui::Rect,
        ln_rect: &egui::Rect,
        scroll: &mut crate::app_state::ScrollState,
        current_cursor_y: Option<f32>,
        ln_text: Option<egui::Color32>,
        ln_active_text: Option<egui::Color32>,
        left_margin: f32,
        line_number_pad_right: f32,
    ) {
        let clip_rect = ui.clip_rect().expand(100.0);
        let mut p = 0;
        let mut is_start_of_para = true;

        for row in &galley.rows {
            let top_y = row.rect().min.y;
            let y = response_rect.min.y + top_y;
            let is_visible = is_start_of_para
                && y <= clip_rect.max.y
                && (y + row.rect().height()) >= clip_rect.min.y;

            if is_visible {
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
                    egui::vec2(left_margin - line_number_pad_right, row.rect().height()),
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

            is_start_of_para = row.ends_with_newline;
            if row.ends_with_newline {
                p += 1;
            }
        }
    }
}
