use egui::Ui;
use egui_commonmark::{CommonMarkCache, EventIteratorItem, Table};
use pulldown_cmark::Alignment;

/* WHY: Uniform vertical padding applied inside every cell (header and body) via apply_alignment.
 * This is the single source of per-row breathing room — grid item_spacing.y is 0, so the
 * CELL_PAD_Y bottom of row N and CELL_PAD_Y top of row N+1 together form the visual gap
 * around each separator line (8px total). */
const CELL_PAD_Y: f32 = 4.0;

pub(crate) struct KatanaTableRendererParts;

impl KatanaTableRendererParts {
    fn header_row_bottom(ui: &Ui) -> f32 {
        /* WHY: grid item_spacing.y is 0, so no half-spacing adjustment is needed.
         * min_rect().bottom() is the exact bottom of the header cell content. */
        ui.min_rect().bottom()
    }

    fn render_mixed_cell<'e>(
        ui: &mut Ui,
        cache: &mut CommonMarkCache,
        items: &[EventIteratorItem<'e>],
        render_cell: &mut dyn FnMut(&mut Ui, &mut CommonMarkCache, &[EventIteratorItem<'e>]),
    ) {
        let mut replaced = false;
        if items.len() == 1 {
            let text = match &items[0].1.0 {
                pulldown_cmark::Event::Text(t) => Some(t.as_ref()),
                pulldown_cmark::Event::Code(t) => Some(t.as_ref()),
                _ => None,
            };
            if let Some(text) = text {
                if text.starts_with("{{os_svg:") && text.ends_with("}}") {
                    const OS_SVG_PREFIX_LEN: usize = "{{os_svg:".len();
                    const OS_SVG_SUFFIX_LEN: usize = "}}".len();
                    let key = &text[OS_SVG_PREFIX_LEN..text.len() - OS_SVG_SUFFIX_LEN];
                    let raw = crate::os_command::OsCommandOps::get(key);
                    crate::widgets::ShortcutWidget::new(&raw).ui(ui);
                    replaced = true;
                }
            }
        }
        if !replaced {
            render_cell(ui, cache, items);
        }
    }

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
        header_bounds: &mut Option<(f32, f32)>,
        header_bottom_y: &mut Option<f32>,
    ) {
        let header_row_top = ui.cursor().min.y;
        for (i, col_w) in col_alloc_width.iter().copied().enumerate().take(num_cols) {
            /* WHY: Capture the boundary between columns (internal separators only). */
            let cell_left_x = ui.cursor().min.x;
            if i > 0 && col_boundaries.len() < num_cols - 1 {
                col_boundaries.push(cell_left_x - ui.spacing().item_spacing.x / 2.0);
            }

            let alignment = alignments.get(i).copied().unwrap_or(Alignment::None);
            Self::apply_alignment(ui, alignment, col_w, |ui| {
                if let Some(hcol) = table_data.header.get(i) {
                    Self::render_mixed_cell(ui, cache, hcol, render_cell);
                }
            });
        }

        let bottom = Self::header_row_bottom(ui);
        *header_bounds = Some((header_row_top, bottom));
        *header_bottom_y = Some(bottom);
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
        _header_bottom_y: Option<f32>,
    ) {
        /* WHY: grid item_spacing.y is 0, so cursor().min.y after end_row() is exactly
         * the bottom of the previous row — no half-spacing gap to account for. */
        let mut current_top_y = ui.cursor().min.y;

        for row in &table_data.rows {
            for (i, row_col) in row.iter().enumerate().take(num_cols) {
                if let Some(&col_w) = col_alloc_width.get(i) {
                    let alignment = alignments.get(i).copied().unwrap_or(Alignment::None);
                    Self::apply_alignment(ui, alignment, col_w, |ui| {
                        Self::render_mixed_cell(ui, cache, row_col, render_cell);
                    });
                } else {
                    /* WHY: Fallback empty label for missing columns, same as pulldown.rs line 1837. */
                    ui.label("");
                }
            }

            /* WHY: grid item_spacing.y is 0 — row bottom is exactly min_rect().bottom(). */
            let current_bottom_y = ui.min_rect().bottom();
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
        let align = match alignment {
            Alignment::Center => egui::Align::Center,
            Alignment::Right => egui::Align::Max,
            _ => egui::Align::Min,
        };
        ui.with_layout(egui::Layout::top_down(align), |ui| {
            /* WHY: item_spacing.y is inherited from the parent grid (10px). Zero it here so
             * that when a cell contains multiple widgets (e.g. plain text + inline code),
             * they stack at natural line height with no extra gap between them. */
            ui.spacing_mut().item_spacing.y = 0.0;
            if width > 0.0 {
                ui.set_min_width(width);
                ui.set_max_width(width);
            }
            ui.add_space(CELL_PAD_Y);
            let result = add_contents(ui);
            ui.add_space(CELL_PAD_Y);
            result
        })
        .inner
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
            let mut h_bounds = None;
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
                    &mut h_bounds,
                    &mut h_bottom_y,
                );
            });

            assert!(h_bounds.is_some());
            assert!(h_bottom_y.is_some());
        });
    }
}
