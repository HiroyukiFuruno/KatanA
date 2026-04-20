use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkOptions, EventIteratorItem, Table};
use pulldown_cmark::Alignment;

use crate::preview_pane::extension_table::decorations::KatanaTableDecorations;
use crate::preview_pane::extension_table::layout::TableLayoutCalculator;
use crate::preview_pane::extension_table::render_parts::KatanaTableRendererParts;

const CHAR_WIDTH_MUL: f32 = 6.0;
const BASE_WIDTH_OFFSET: f32 = 16.0;
/* WHY: Horizontal item spacing between columns — unchanged from upstream reference. */
const ITEM_SPACING: f32 = 10.0;
const DEFAULT_MARGIN: i8 = 5;
const MIN_COL_WIDTH: f32 = 10.0;
const TABLE_WIDTH_PADDING: f32 = 22.0;
/* WHY: top_down scope_builder is required so add_space adds VERTICAL space.
 * In the outer left_to_right+main_wrap layout, add_space would be horizontal. */
const TABLE_OUTER_MARGIN: f32 = 6.0;

pub struct KatanaTableRenderer;

#[cfg(test)]
#[path = "renderer_tests.rs"]
mod renderer_tests;

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

        let safe_width = parent_available_width;
        /* WHY: Match upstream sizing contract so table frame + inner content never overflow panel. */
        let table_width = (safe_width - TABLE_WIDTH_PADDING).max(0.0);
        let col_max_chars = TableLayoutCalculator::calculate_col_max_chars(&table_data, num_cols);
        let ideal_w_and_index = TableLayoutCalculator::compute_ideal_widths(
            &col_max_chars,
            CHAR_WIDTH_MUL,
            BASE_WIDTH_OFFSET,
        );
        /* WHY: egui::Grid contributes one extra horizontal spacing slot in this layout path.
         * Subtract num_cols * item_spacing (not num_cols - 1) so the table never requests
         * parent_width + ITEM_SPACING and ratchets a resizable parent wider frame-by-frame. */
        let available_w =
            (parent_available_width - TABLE_WIDTH_PADDING - (num_cols as f32) * ITEM_SPACING)
                .max(0.0);
        let col_alloc_width =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);

        let cursor = ui.next_widget_position();
        let outer_rect = egui::Rect::from_min_max(
            cursor,
            egui::pos2(
                cursor.x + safe_width,
                ui.max_rect().max.y.max(cursor.y + 1.0),
            ),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(outer_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
            |constrained_ui| {
                constrained_ui.add_space(TABLE_OUTER_MARGIN);
                constrained_ui.set_max_width(safe_width);

                let scroll_output =
                    egui::ScrollArea::horizontal()
                        .id_salt(table_id.with("scroll"))
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .auto_shrink([false, true])
                        .min_scrolled_width(0.0)
                        .show(constrained_ui, |inner_ui| {
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

                            let mut header_bounds = None;
                            let mut header_bottom_y = None;
                            let mut col_boundaries = Vec::new();
                            let mut row_bounds = Vec::new();

                            grid_ui.spacing_mut().item_spacing.x = ITEM_SPACING;
                            /* WHY: Row-to-row gap is 0 — visual separation between rows
                             * comes from CELL_PAD_Y (top+bottom per cell) in apply_alignment,
                             * so rows share their padding naturally at each separator line.
                             * A non-zero grid item_spacing.y would create an empty "gap strip"
                             * between the separator line and the next row's content. */
                            grid_ui.spacing_mut().item_spacing.y = 0.0;

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
                                        &mut header_bounds,
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
                                header_bounds,
                                col_boundaries,
                                row_bg_indices,
                                row_bounds,
                            )
                        });

                            let (
                                header_bg_idx,
                                header_bounds,
                                col_boundaries,
                                row_bg_indices,
                                row_bounds,
                            ) = frame_res.inner;

                            /* WHY: Pass the Frame's own rect directly — no top/bottom shrink.
                             * Frame::inner_margin provides natural padding on all sides, and
                             * CELL_PAD_Y inside apply_alignment handles per-row breathing room.
                             * Any additional offset here would de-sync the border from the fill. */
                            KatanaTableDecorations::draw_decorations(
                                inner_ui,
                                frame_res.response.rect,
                                header_bg_idx,
                                header_bounds,
                                &col_boundaries,
                                table_data.rows.len(),
                                &row_bg_indices,
                                &row_bounds,
                            );

                            frame_res.response
                        });

                let mut out_res = scroll_output.inner;
                /* WHY: Clamp only for wide tables (horizontal scroll). For normal tables,
                 * preserve the actual frame rect so table_hook.rs highlight matches the
                 * drawn border rect used by draw_decorations exactly. */
                let visible_rect = constrained_ui.clip_rect();
                if out_res.rect.width() > visible_rect.width() {
                    out_res.rect = egui::Rect::from_min_size(
                        out_res.rect.min,
                        egui::vec2(visible_rect.width(), out_res.rect.height()),
                    );
                }
                constrained_ui.add_space(TABLE_OUTER_MARGIN);
                out_res
            },
        )
        .inner
    }
}
