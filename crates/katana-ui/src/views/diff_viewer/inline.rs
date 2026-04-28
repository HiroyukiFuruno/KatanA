use super::row::DiffViewerRowOps;
use super::style::DiffViewerPalette;
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
                    Self::show_row(ui, file, row, code_width, &palette);
                }
            });
    }

    fn show_row(
        ui: &mut egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        row: &crate::diff_review::InlineDiffRow,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        match row {
            crate::diff_review::InlineDiffRow::Line(line) => {
                Self::show_line(ui, line, code_width, palette);
                ui.end_row();
            }
            crate::diff_review::InlineDiffRow::Collapsed(block) => {
                let expanded = Self::is_expanded(ui, file, block);
                if !expanded {
                    if Self::show_collapsed_row(ui, block, code_width, palette) {
                        Self::toggle_expanded(ui, file, block, expanded);
                    }
                    ui.end_row();
                    return;
                }

                Self::show_expanded_block(ui, file, block, code_width, palette);
            }
        }
    }

    fn show_expanded_block(
        ui: &mut egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        for (index, line) in block.lines.iter().enumerate() {
            Self::show_expanded_block_line(ui, file, block, index, line, code_width, palette);
        }
    }

    fn show_expanded_block_line(
        ui: &mut egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
        index: usize,
        line: &crate::diff_review::DiffLine,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        if index == 0 {
            Self::show_first_expanded_block_line(ui, file, block, line, code_width, palette);
            return;
        }

        Self::show_line(ui, line, code_width, palette);
        ui.end_row();
    }

    fn show_first_expanded_block_line(
        ui: &mut egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
        line: &crate::diff_review::DiffLine,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        let expanded = true;
        if Self::show_expanded_line(ui, line, code_width, palette) {
            Self::toggle_expanded(ui, file, block, expanded);
        }
    }

    fn show_line(
        ui: &mut egui::Ui,
        line: &crate::diff_review::DiffLine,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        let tone = DiffViewerRowOps::tone_for(line.kind);
        DiffViewerRowOps::sign_cell(ui, line.kind, palette);
        DiffViewerRowOps::line_number_cell(
            ui,
            line.before_line_number.or(line.after_line_number),
            tone,
            palette,
        );
        DiffViewerRowOps::code_cell(
            ui,
            &line.text,
            code_width,
            tone,
            palette,
            &line.highlight_ranges,
        );
    }

    fn show_collapsed_row(
        ui: &mut egui::Ui,
        block: &crate::diff_review::UnchangedBlock,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) -> bool {
        DiffViewerRowOps::sign_cell(ui, crate::diff_review::DiffLineKind::Unchanged, palette);
        let icon_clicked = DiffViewerRowOps::collapsed_toggle_cell(ui, palette, false);
        let text_clicked =
            DiffViewerRowOps::collapsed_text_cell(ui, block.line_count, code_width, palette)
                .clicked();
        icon_clicked || text_clicked
    }

    fn show_expanded_line(
        ui: &mut egui::Ui,
        line: &crate::diff_review::DiffLine,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) -> bool {
        let tone = DiffViewerRowOps::tone_for(line.kind);
        DiffViewerRowOps::sign_cell(ui, line.kind, palette);
        let toggle_clicked = DiffViewerRowOps::line_number_toggle_cell(
            ui,
            line.before_line_number.or(line.after_line_number),
            palette,
        );
        DiffViewerRowOps::code_cell(
            ui,
            &line.text,
            code_width,
            tone,
            palette,
            &line.highlight_ranges,
        );
        ui.end_row();
        toggle_clicked
    }

    fn is_expanded(
        ui: &egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
    ) -> bool {
        ui.ctx()
            .data(|data| data.get_temp::<bool>(Self::block_id(file, block)))
            .unwrap_or(false)
    }

    fn toggle_expanded(
        ui: &egui::Ui,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
        expanded: bool,
    ) {
        ui.ctx()
            .data_mut(|data| data.insert_temp(Self::block_id(file, block), !expanded));
    }

    fn block_id(
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
    ) -> egui::Id {
        egui::Id::new((
            "diff_viewer_unchanged_block",
            file.path.as_path(),
            block.before_start_line_number,
            block.after_start_line_number,
            block.line_count,
        ))
    }
}
