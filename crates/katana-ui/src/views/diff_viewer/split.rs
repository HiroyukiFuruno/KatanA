use super::row::DiffViewerRowOps;
use super::style::DiffViewerPalette;
use eframe::egui;

const SPLIT_GRID_COLUMNS: usize = 4;
const GRID_SPACING_X: f32 = 2.0;
const GRID_SPACING_Y: f32 = 1.0;

pub(super) struct DiffViewerSplitOps;

impl DiffViewerSplitOps {
    pub(super) fn show(ui: &mut egui::Ui, file: &crate::diff_review::DiffReviewFile) {
        let palette = DiffViewerPalette::from_ui(ui);
        let code_width = DiffViewerRowOps::split_code_width(ui);
        egui::Grid::new(ui.id().with("diff_viewer_split_grid"))
            .num_columns(SPLIT_GRID_COLUMNS)
            .spacing(egui::vec2(GRID_SPACING_X, GRID_SPACING_Y))
            .striped(false)
            .show(ui, |ui| {
                for row in &file.model.split_rows {
                    match row {
                        crate::diff_review::SplitDiffRow::Line(line) => {
                            DiffViewerRowOps::show_split_cell(
                                ui,
                                line.before.as_ref(),
                                code_width,
                                &palette,
                            );
                            DiffViewerRowOps::show_split_cell(
                                ui,
                                line.after.as_ref(),
                                code_width,
                                &palette,
                            );
                        }
                        crate::diff_review::SplitDiffRow::Collapsed(block) => {
                            DiffViewerRowOps::show_collapsed_side(
                                ui,
                                block.line_count,
                                code_width,
                                &palette,
                            );
                            DiffViewerRowOps::show_collapsed_side(
                                ui,
                                block.line_count,
                                code_width,
                                &palette,
                            );
                        }
                    }
                    ui.end_row();
                }
            });
    }
}
