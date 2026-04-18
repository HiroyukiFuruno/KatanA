use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkOptions, EventIteratorItem, Table};
use pulldown_cmark::Alignment;

use crate::preview_pane::extension_table::decorations::KatanaTableDecorations;
use crate::preview_pane::extension_table::layout::TableLayoutCalculator;
use crate::preview_pane::extension_table::render_parts::KatanaTableRendererParts;

/* WHY: Layout Constants - Aligned with pulldown.rs workspace reference */
const CHAR_WIDTH_MUL: f32 = 6.0;
const BASE_WIDTH_OFFSET: f32 = 16.0;
const ITEM_SPACING: f32 = 10.0;
const DEFAULT_MARGIN: i8 = 5;
const TABLE_VERTICAL_SPACING: f32 = 5.0;
const MIN_COL_WIDTH: f32 = 10.0;

pub struct KatanaTableRenderer;

impl KatanaTableRenderer {
    pub fn render<'e>(
        ui: &mut Ui,
        cache: &mut CommonMarkCache,
        _render_cell_options: &CommonMarkOptions,
        table_data: Table<'e>,
        alignments: &[Alignment],
        max_width: f32,
        render_cell: &mut dyn FnMut(&mut Ui, &mut CommonMarkCache, &[EventIteratorItem<'e>]),
    ) -> egui::Response {
        let num_cols = table_data.header.len().max(1);
        let id = ui.id().with("table_grid");

        /* WHY: available_width() in SplitView might be constrained by parent max_rect. */
        let parent_available_width = ui.available_width().min(max_width);

        /* WHY: Add vertical separation: 5px as requested. */
        ui.add_space(TABLE_VERTICAL_SPACING);

        let mut col_boundaries = vec![];
        let mut header_bottom_y = None;
        let mut row_bounds = vec![];

        /* WHY: Calculate column widths based on the REMOTE logic (pulldown.rs reference). */
        let table_width = (parent_available_width - (DEFAULT_MARGIN as f32) * 2.0).max(0.0);
        let col_max_chars = TableLayoutCalculator::calculate_col_max_chars(&table_data, num_cols);
        let ideal_w_and_index = TableLayoutCalculator::compute_ideal_widths(
            &col_max_chars,
            CHAR_WIDTH_MUL,
            BASE_WIDTH_OFFSET,
        );
        let available_w = (table_width - (num_cols as f32 - 1.0) * ITEM_SPACING).max(0.0);
        let col_alloc_width =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);

        /* WHY: SYSTEMIC ARCHITECTURAL CONSTRAINT
        We MUST wrap the child rendering in a strictly constrained scope.
        Egui's ScrollArea uses `auto_shrink([false, true])` to span fully, which natively attempts to
        grab `ui.available_width()`. If we do not restrict `ui.available_width()` explicitly using a scope,
        the ScrollArea grabs the entire unconstrained parent bounds (e.g. 500px in tests, or full screen),
        bypassing the `max_width` parameter entirely and permanently locking layout expansion. */
        let safe_width = ui.available_width().min(max_width);
        let table_width = (safe_width - (DEFAULT_MARGIN as f32) * 2.0).max(0.0);

        let scroll_res = ui
            .scope(|constrained_ui| {
                constrained_ui.set_max_width(safe_width);

                let scroll_output = egui::ScrollArea::horizontal()
                    .id_salt(id.with("table_scroll"))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([true, false]) // Shrinking horizontally is fine, but vertical shouldn't be affected by tables.
                    .min_scrolled_width(0.0) // CRITICAL: Stop the ScrollArea from reporting the stretched table width as its own minimum!
                    .show(constrained_ui, |inner_ui| {
                        /* WHY: Specify the known computed widths.
                        MUST NOT call set_min_width here - inside a ScrollArea it would push
                        the parent panel's min_rect wider each frame (Ratchet Bug). */
                        inner_ui.set_max_width(table_width);

                        let frame_res = egui::Frame::none()
                            .inner_margin(egui::Margin::same(DEFAULT_MARGIN))
                            .show(inner_ui, |grid_ui| {
                                grid_ui.spacing_mut().item_spacing =
                                    egui::vec2(ITEM_SPACING, ITEM_SPACING);

                                egui::Grid::new(id)
                                    .num_columns(num_cols)
                                    .striped(true)
                                    .min_col_width(MIN_COL_WIDTH)
                                    .show(grid_ui, |grid_ui| {
                                        KatanaTableRendererParts::render_header(
                                            grid_ui,
                                            cache,
                                            &table_data,
                                            alignments,
                                            &mut col_boundaries,
                                            &col_alloc_width,
                                            render_cell,
                                            num_cols,
                                            &mut header_bottom_y,
                                        );
                                        KatanaTableRendererParts::render_body(
                                            grid_ui,
                                            cache,
                                            &table_data,
                                            alignments,
                                            &col_alloc_width,
                                            render_cell,
                                            num_cols,
                                            &mut row_bounds,
                                            header_bottom_y,
                                        );
                                    });
                            });

                        /* WHY: Apply decorations tracking coordinate system of inner_ui */
                        KatanaTableDecorations::draw_decorations(
                            inner_ui,
                            frame_res.response.rect,
                            None,
                            header_bottom_y,
                            &col_boundaries,
                            table_data.rows.len(),
                            &[],
                            &row_bounds,
                        );

                        /* Return frame response representing the grid bounds */
                        frame_res.response
                    });

                let mut res = scroll_output.inner;
                res.rect = scroll_output.inner_rect;
                res
            })
            .inner;

        /* WHY: Add closing vertical separation outside the horizontal scope boundary */
        ui.add_space(TABLE_VERTICAL_SPACING);

        scroll_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, RawInput, pos2, vec2};

    #[test]
    fn test_renderer_sanity() {
        let ctx = Context::default();
        let _ = ctx.run(RawInput::default(), |ctx| {
            let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0));
            let builder = egui::UiBuilder::new().max_rect(rect);
            let mut ui = Ui::new(ctx.clone(), egui::Id::new("test"), builder);

            let mut render_cell = |_ui: &mut Ui, _cache: &mut CommonMarkCache, _items: &[_]| {};
            let table_data = Table {
                header: vec![],
                rows: vec![],
            };

            let res = KatanaTableRenderer::render(
                &mut ui,
                &mut CommonMarkCache::default(),
                &CommonMarkOptions::default(),
                table_data,
                &[],
                400.0,
                &mut render_cell,
            );
            /* WHY: Verification - Resulting width MUST be within parent constraint. */
            assert!(res.rect.width() <= 400.0 + 1.0); // Allow small floating point epsilon
        });
    }

    #[test]
    fn test_renderer_long_content_no_expand() {
        let ctx = Context::default();
        let _ = ctx.run(RawInput::default(), |ctx| {
            let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0));
            let builder = egui::UiBuilder::new().max_rect(rect);
            let mut ui = Ui::new(ctx.clone(), egui::Id::new("test_long"), builder);

            let mut render_cell = |ui: &mut Ui, _cache: &mut CommonMarkCache, _items: &[_]| {
                /* Simulate a cell that is very wide (e.g., 2000px). */
                ui.allocate_space(vec2(2000.0, 20.0));
            };

            /* Give it 3 columns so the table wants to be at least 6000px wide. */
            let table_data = Table {
                header: vec![vec![], vec![], vec![]],
                rows: vec![],
            };

            let parent_max_width = 400.0;
            let _res = KatanaTableRenderer::render(
                &mut ui,
                &mut CommonMarkCache::default(),
                &CommonMarkOptions::default(),
                table_data,
                &[],
                parent_max_width,
                &mut render_cell,
            );

            /* WHY: Verification - Resulting UI min_rect MUST be within parent constraint.
            The returned response rect tracks the inner content size, so it CAN be larger.
            We exclusively check the UI's bounding box to ensure layout won't push the splitter. */
            assert!(
                ui.min_rect().width() <= parent_max_width + 1.0,
                "Layout expansion detected in UI! UI min_rect width of {}, max_width was {}",
                ui.min_rect().width(),
                parent_max_width
            );
        });
    }
}
