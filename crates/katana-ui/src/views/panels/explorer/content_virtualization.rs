use super::content::ExplorerContent;
use super::tree_entry::TreeEntryNode;
use crate::shell::TREE_ROW_HEIGHT;
use crate::shell_ui::TreeRenderContext;
use crate::state::explorer_projection::{ExplorerProjectionCache, ExplorerProjectionRow};
use eframe::egui;

impl ExplorerContent<'_> {
    pub(super) fn show_virtualized_tree(
        ui: &mut egui::Ui,
        tree: &[katana_core::workspace::TreeEntry],
        rows: &[ExplorerProjectionRow],
        ctx: &mut TreeRenderContext,
        ws_root: &std::path::Path,
    ) -> usize {
        let row_stride = TREE_ROW_HEIGHT + ui.spacing().item_spacing.y;
        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show_viewport(ui, |ui, viewport| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let visible_rows =
                    Self::show_visible_rows(ui, viewport, tree, rows, ctx, row_stride);
                Self::show_root_drop_area(ui, viewport, rows.len(), ctx, ws_root, row_stride);
                visible_rows
            })
            .inner
    }

    fn show_visible_rows(
        ui: &mut egui::Ui,
        viewport: egui::Rect,
        tree: &[katana_core::workspace::TreeEntry],
        rows: &[ExplorerProjectionRow],
        ctx: &mut TreeRenderContext,
        row_stride: f32,
    ) -> usize {
        ui.set_height(rows.len() as f32 * row_stride + TREE_ROW_HEIGHT * 2.0);
        let range = ExplorerProjectionCache::visible_row_range(
            viewport.min.y..viewport.max.y,
            row_stride,
            rows.len(),
        );
        let visible_count = range.len();
        let top = ui.max_rect().top() + range.start as f32 * row_stride;
        let height = range.len() as f32 * row_stride;
        let rect = egui::Rect::from_min_size(
            egui::pos2(ui.max_rect().left(), top),
            egui::vec2(ui.max_rect().width(), height),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |visible_ui| {
            visible_ui.skip_ahead_auto_ids(range.start);
            for row in &rows[range] {
                Self::show_projected_row(visible_ui, tree, row, ctx);
            }
        });
        visible_count
    }

    fn show_projected_row(
        ui: &mut egui::Ui,
        tree: &[katana_core::workspace::TreeEntry],
        row: &ExplorerProjectionRow,
        ctx: &mut TreeRenderContext,
    ) {
        let Some(entry) = row.resolve(tree) else {
            ui.allocate_space(egui::vec2(ui.available_width(), TREE_ROW_HEIGHT));
            return;
        };
        ctx.depth = row.depth;
        let top = ui.cursor().top();
        TreeEntryNode::new(entry, ctx).show(ui);
        if ctx.show_vertical_line {
            Self::paint_vertical_lines(ui, row, top);
        }
    }

    fn paint_vertical_lines(ui: &egui::Ui, row: &ExplorerProjectionRow, top: f32) {
        let rect = egui::Rect::from_min_size(
            egui::pos2(ui.max_rect().left(), top),
            egui::vec2(ui.max_rect().width(), TREE_ROW_HEIGHT),
        );
        for depth in 0..row.depth {
            super::dir_entry_paint::DirectoryEntryPaintOps::paint_vertical_line(
                ui,
                rect,
                rect.top(),
                rect.bottom(),
                depth,
            );
        }
    }

    fn show_root_drop_area(
        ui: &mut egui::Ui,
        viewport: egui::Rect,
        row_count: usize,
        ctx: &mut TreeRenderContext,
        ws_root: &std::path::Path,
        row_stride: f32,
    ) {
        let top = row_count as f32 * row_stride;
        let height = TREE_ROW_HEIGHT * 2.0;
        if viewport.max.y < top || viewport.min.y > top + height {
            return;
        }
        let rect = egui::Rect::from_min_size(
            egui::pos2(ui.max_rect().left(), ui.max_rect().top() + top),
            egui::vec2(ui.max_rect().width(), height),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |root_ui| {
            super::root_drop_area::ExplorerRootDropArea::show(root_ui, ctx, ws_root);
        });
    }
}
