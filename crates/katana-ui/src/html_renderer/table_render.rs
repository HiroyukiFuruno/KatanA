use eframe::egui;
use katana_core::html::{HtmlNode, LinkAction};

use super::HtmlRenderer;
use super::collect_text;

const TABLE_MIN_CELL_WIDTH: f32 = 120.0;
const TABLE_MAX_CELL_WIDTH: f32 = 320.0;

impl<'a> HtmlRenderer<'a> {
    pub(super) fn render_table(
        &mut self,
        headers: &[Vec<HtmlNode>],
        rows: &[Vec<Vec<HtmlNode>>],
    ) -> Option<LinkAction> {
        let mut action = None;
        let table_id = self.ui.next_auto_id().with("html_table");
        let text_color = self.text_color;
        let max_image_width = self.max_image_width;
        let column_count = Self::table_column_count(headers, rows).max(1);
        let available_width = self.ui.available_width().max(1.0);
        let cell_width = (available_width / column_count as f32)
            .clamp(TABLE_MIN_CELL_WIDTH, TABLE_MAX_CELL_WIDTH);
        egui::Grid::new(table_id)
            .striped(true)
            .min_col_width(cell_width)
            .show(self.ui, |ui| {
                Self::render_table_headers(ui, headers, cell_width);
                for row in rows {
                    Self::render_table_row(
                        ui,
                        row,
                        cell_width,
                        text_color,
                        max_image_width,
                        &mut action,
                    );
                    ui.end_row();
                }
            });
        action
    }

    fn render_table_headers(ui: &mut egui::Ui, headers: &[Vec<HtmlNode>], cell_width: f32) {
        if headers.is_empty() {
            return;
        }
        for header in headers {
            let header_text = collect_text(header);
            ui.add_sized(
                egui::vec2(cell_width, 0.0),
                egui::Label::new(egui::RichText::new(header_text).strong()).wrap(),
            );
        }
        ui.end_row();
    }

    fn render_table_row(
        ui: &mut egui::Ui,
        row: &[Vec<HtmlNode>],
        cell_width: f32,
        text_color: Option<egui::Color32>,
        max_image_width: f32,
        action: &mut Option<LinkAction>,
    ) {
        for cell in row {
            Self::render_table_cell(ui, cell, cell_width, text_color, max_image_width, action);
        }
    }

    fn render_table_cell(
        ui: &mut egui::Ui,
        cell: &[HtmlNode],
        cell_width: f32,
        text_color: Option<egui::Color32>,
        max_image_width: f32,
        action: &mut Option<LinkAction>,
    ) {
        ui.allocate_ui_with_layout(
            egui::vec2(cell_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(cell_width);
                let mut inner = HtmlRenderer::new_inner(ui, text_color, max_image_width);
                if let Some(cell_action) = inner.render_nodes(cell) {
                    *action = Some(cell_action);
                }
            },
        );
    }

    fn table_column_count(headers: &[Vec<HtmlNode>], rows: &[Vec<Vec<HtmlNode>>]) -> usize {
        rows.iter()
            .map(Vec::len)
            .chain(std::iter::once(headers.len()))
            .max()
            .unwrap_or(0)
    }
}
