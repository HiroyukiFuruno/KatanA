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
const TABLE_WIDTH_PADDING: f32 = 22.0;

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
        /* WHY: Each table instance must use a distinct id to avoid cross-table layout state bleed. */
        let table_id = ui.next_auto_id().with("table");

        /* WHY: available_width() in SplitView might be constrained by parent max_rect. */
        let parent_available_width = ui.available_width().min(max_width);
        ui.add_space(TABLE_VERTICAL_SPACING);

        let safe_width = parent_available_width;
        /* WHY: Match upstream sizing contract so table frame + inner content never overflow panel. */
        let table_width = (safe_width - TABLE_WIDTH_PADDING).max(0.0);
        let col_max_chars = TableLayoutCalculator::calculate_col_max_chars(&table_data, num_cols);
        let ideal_w_and_index = TableLayoutCalculator::compute_ideal_widths(
            &col_max_chars,
            CHAR_WIDTH_MUL,
            BASE_WIDTH_OFFSET,
        );
        let available_w = (table_width - (num_cols as f32 - 1.0) * ITEM_SPACING).max(0.0);
        let col_alloc_width =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);

        let scroll_output = ui.scope(|constrained_ui| {
            constrained_ui.set_max_width(safe_width);

            egui::ScrollArea::horizontal()
                .id_salt(table_id.with("scroll"))
                .auto_shrink([true, true]) // MUST shrink vertically to avoid expanding the panel.
                .min_scrolled_width(0.0)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(constrained_ui, |inner_ui| {
                    /* WHY: We MUST clamp the inner UI so that the table does not
                    request an infinite or expanding width across frames. */
                    inner_ui.set_max_width(table_width);

                    let frame_res = egui::Frame::none()
                        .inner_margin(egui::Margin::same(DEFAULT_MARGIN))
                        .show(inner_ui, |grid_ui| {
                            /* WHY: Create placeholders for backgrounds BEFORE grid rendering,
                            so they are drawn behind the text in the same coordinate space. */
                            let header_bg_idx = Some(grid_ui.painter().add(egui::Shape::Noop));
                            let mut row_bg_indices = Vec::new();
                            for row_idx in 0..table_data.rows.len() {
                                if row_idx % 2 == 1 {
                                    row_bg_indices
                                        .push((row_idx, grid_ui.painter().add(egui::Shape::Noop)));
                                }
                            }

                            let mut header_top_y = None;
                            let mut header_bottom_y = None;
                            let mut col_boundaries = Vec::new();
                            let mut row_bounds = Vec::new();

                            grid_ui.spacing_mut().item_spacing.x = ITEM_SPACING;
                            grid_ui.spacing_mut().item_spacing.y = ITEM_SPACING;

                            egui::Grid::new(table_id.with("grid"))
                                .num_columns(num_cols)
                                .striped(false) // Handle striping manually via KatanaTableDecorations
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
                                        &mut header_top_y,
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

                            (
                                header_bg_idx,
                                header_top_y,
                                header_bottom_y,
                                col_boundaries,
                                row_bg_indices,
                                row_bounds,
                            )
                        });

                    let (
                        header_bg_idx,
                        header_top_y,
                        header_bottom_y,
                        col_boundaries,
                        row_bg_indices,
                        row_bounds,
                    ) = frame_res.inner;

                    /* WHY: Apply decorations tracking coordinate system of inner_ui */
                    KatanaTableDecorations::draw_decorations(
                        inner_ui,
                        frame_res
                            .response
                            .rect
                            .shrink2(egui::vec2(0.0, TABLE_VERTICAL_SPACING)),
                        header_bg_idx,
                        header_top_y,
                        header_bottom_y,
                        &col_boundaries,
                        table_data.rows.len(),
                        &row_bg_indices,
                        &row_bounds,
                    );

                    frame_res.response
                })
        });

        /* WHY: Return the unaltered ScrollArea response, which respects max_width bounding.
        DO NOT override .rect = scroll_output.inner.inner_rect, because inner_rect is
        the full unclipped content width (e.g. 2600px). Returning it causes the parent layout
        to expand and ratchetting to occur! scroll_output.response is the clipped visible bounds. */
        ui.add_space(TABLE_VERTICAL_SPACING);
        scroll_output.response
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
