use super::row::DiffViewerRowOps;
use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

const INLINE_GRID_COLUMNS: usize = 3;
const GRID_SPACING_X: f32 = 2.0;
const GRID_SPACING_Y: f32 = 1.0;

pub(super) struct DiffViewerInlineOps;

impl DiffViewerInlineOps {
    pub(super) fn show(ui: &mut egui::Ui, file: &crate::diff_review::DiffReviewFile) {
        let palette = DiffViewerPalette::from_ui(ui);
        let code_width = DiffViewerRowOps::inline_code_width(ui);
        egui::Grid::new(ui.id().with("diff_viewer_inline_grid"))
            .num_columns(INLINE_GRID_COLUMNS)
            .spacing(egui::vec2(GRID_SPACING_X, GRID_SPACING_Y))
            .striped(false)
            .show(ui, |ui| {
                for row in &file.model.inline_rows {
                    Self::show_row(ui, row, code_width, &palette);
                    ui.end_row();
                }
            });
    }

    fn show_row(
        ui: &mut egui::Ui,
        row: &crate::diff_review::InlineDiffRow,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        match row {
            crate::diff_review::InlineDiffRow::Line(line) => {
                let tone = DiffViewerRowOps::tone_for(line.kind);
                DiffViewerRowOps::sign_cell(ui, line.kind, palette);
                DiffViewerRowOps::line_number_cell(
                    ui,
                    line.before_line_number.or(line.after_line_number),
                    tone,
                    palette,
                );
                DiffViewerRowOps::code_cell(ui, &line.text, code_width, tone, palette);
            }
            crate::diff_review::InlineDiffRow::Collapsed(block) => {
                DiffViewerRowOps::sign_cell(
                    ui,
                    crate::diff_review::DiffLineKind::Unchanged,
                    palette,
                );
                DiffViewerRowOps::line_number_cell(
                    ui,
                    Some(block.before_start_line_number),
                    DiffTone::Collapsed,
                    palette,
                );
                DiffViewerRowOps::collapsed_text_cell(ui, block.line_count, code_width, palette);
            }
        }
    }
}
