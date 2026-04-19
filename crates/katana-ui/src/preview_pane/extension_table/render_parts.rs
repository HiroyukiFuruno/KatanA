use egui::Ui;
use egui_commonmark::{CommonMarkCache, EventIteratorItem, Table};
use pulldown_cmark::Alignment;

pub(crate) struct KatanaTableRendererParts;

impl KatanaTableRendererParts {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_header<'e>(
        ui: &mut Ui,
        cache: &mut CommonMarkCache,
        table_data: &Table<'e>,
        alignments: &[Alignment],
        col_boundaries: &mut Vec<f32>,
        col_alloc_width: &[f32],
        render_cell: &mut dyn FnMut(&mut Ui, &mut CommonMarkCache, &[EventIteratorItem<'e>]),
        num_cols: usize,
        header_top_y: &mut Option<f32>,
        header_bottom_y: &mut Option<f32>,
    ) {
        *header_top_y = Some(ui.cursor().min.y);
        let header_vertical_padding = ui.spacing().item_spacing.y / 2.0;
        for (i, col_w) in col_alloc_width.iter().copied().enumerate().take(num_cols) {
            /* WHY: Capture the boundary between columns (internal separators only), matching pulldown.rs. */
            let cell_left_x = ui.cursor().min.x;
            if i > 0 && col_boundaries.len() < num_cols - 1 {
                col_boundaries.push(cell_left_x - ui.spacing().item_spacing.x / 2.0);
            }

            let alignment = alignments.get(i).copied().unwrap_or(Alignment::None);
            Self::apply_alignment(ui, alignment, col_w, |ui| {
                ui.add_space(header_vertical_padding);
                if let Some(hcol) = table_data.header.get(i) {
                    render_cell(ui, cache, hcol);
                }
            });
        }

        /* WHY: Capture header_bottom_y BEFORE end_row() — matches pulldown.rs reference line 1789-1792. */
        let h_bottom = ui.min_rect().bottom() + ui.spacing().item_spacing.y / 2.0;
        *header_bottom_y = Some(h_bottom);
        ui.end_row();
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_body<'e>(
        ui: &mut Ui,
        cache: &mut CommonMarkCache,
        table_data: &Table<'e>,
        alignments: &[Alignment],
        col_alloc_width: &[f32],
        render_cell: &mut dyn FnMut(&mut Ui, &mut CommonMarkCache, &[EventIteratorItem<'e>]),
        num_cols: usize,
        row_bounds: &mut Vec<(f32, f32)>,
        header_bottom_y: Option<f32>,
    ) {
        /* WHY: Match pulldown.rs reference: current_top_y starts from header_bottom_y
         * and is carried forward between rows (line 1795). */
        let mut current_top_y = header_bottom_y.unwrap_or_else(|| ui.min_rect().bottom());

        for row in &table_data.rows {
            for (i, row_col) in row.iter().enumerate().take(num_cols) {
                if let Some(&col_w) = col_alloc_width.get(i) {
                    let alignment = alignments.get(i).copied().unwrap_or(Alignment::None);
                    Self::apply_alignment(ui, alignment, col_w, |ui| {
                        render_cell(ui, cache, row_col);
                    });
                } else {
                    /* WHY: Fallback empty label for missing columns, same as pulldown.rs line 1837. */
                    ui.label("");
                }
            }

            /* WHY: Capture row_bottom_y BEFORE end_row() — matches pulldown.rs reference line 1840-1845. */
            let current_bottom_y = ui.min_rect().bottom() + ui.spacing().item_spacing.y / 2.0;
            row_bounds.push((current_top_y, current_bottom_y));
            current_top_y = current_bottom_y;

            ui.end_row();
        }
    }

    pub(crate) fn apply_alignment<R>(
        ui: &mut Ui,
        alignment: Alignment,
        width: f32,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        match alignment {
            Alignment::Center => {
                let layout = egui::Layout::top_down(egui::Align::Center);
                ui.with_layout(layout, |ui| {
                    if width > 0.0 {
                        ui.set_min_width(width);
                        ui.set_max_width(width);
                    }
                    add_contents(ui)
                })
                .inner
            }
            Alignment::Right => {
                let layout = egui::Layout::top_down(egui::Align::Max);
                ui.with_layout(layout, |ui| {
                    if width > 0.0 {
                        ui.set_min_width(width);
                        ui.set_max_width(width);
                    }
                    add_contents(ui)
                })
                .inner
            }
            _ => {
                let layout = egui::Layout::top_down(egui::Align::Min);
                ui.with_layout(layout, |ui| {
                    if width > 0.0 {
                        ui.set_min_width(width);
                        ui.set_max_width(width);
                    }
                    add_contents(ui)
                })
                .inner
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, RawInput, pos2, vec2};

    #[test]
    fn test_header_bottom_y_capture() {
        let ctx = Context::default();
        let _ = ctx.run(RawInput::default(), |ctx| {
            let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 200.0));
            let builder = egui::UiBuilder::new().max_rect(rect);
            let mut ui = egui::Ui::new(ctx.clone(), egui::Id::new("test"), builder);

            let mut boundaries = vec![];
            let mut h_top_y = None;
            let mut h_bottom_y = None;
            let mut render_cell =
                |_ui: &mut egui::Ui, _cache: &mut CommonMarkCache, _items: &[_]| {};

            let table_data = Table {
                header: vec![vec![]],
                rows: vec![],
            };

            egui::Grid::new("test_grid").show(&mut ui, |ui| {
                KatanaTableRendererParts::render_header(
                    ui,
                    &mut CommonMarkCache::default(),
                    &table_data,
                    &[Alignment::None],
                    &mut boundaries,
                    &[100.0],
                    &mut render_cell,
                    1,
                    &mut h_top_y,
                    &mut h_bottom_y,
                );
            });

            assert!(h_top_y.is_some());
            assert!(h_bottom_y.is_some());
        });
    }
}
