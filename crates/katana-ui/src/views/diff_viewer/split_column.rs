use super::row::DiffViewerRowOps;
use super::split::SplitSide;
use super::split_state::DiffViewerSplitStateOps;
use super::style::DiffViewerPalette;
use eframe::egui;

pub(super) struct DiffViewerSplitColumnOps;

impl DiffViewerSplitColumnOps {
    pub(super) fn show(
        ui: &mut egui::Ui,
        params: SplitColumnParams<'_>,
        pending_toggles: &mut Vec<egui::Id>,
    ) -> f32 {
        let code_width = DiffViewerRowOps::split_code_width(ui);
        let mut area = egui::ScrollArea::horizontal()
            .id_salt((params.area_id, params.file.path.as_path()))
            .auto_shrink([false, false]);
        if params.previous_offset > 0.0 {
            area = area.horizontal_scroll_offset(params.previous_offset);
        }

        let out = area.show(ui, |ui_inner| {
            for row in &params.file.model.split_rows {
                Self::show_row(ui_inner, row, &params, code_width, pending_toggles);
            }
        });
        out.state.offset.x
    }

    fn show_row(
        ui: &mut egui::Ui,
        row: &crate::diff_review::SplitDiffRow,
        params: &SplitColumnParams<'_>,
        code_width: f32,
        pending_toggles: &mut Vec<egui::Id>,
    ) {
        match row {
            crate::diff_review::SplitDiffRow::Line(line) => {
                Self::show_line(ui, line, params.side, code_width, params.palette);
            }
            crate::diff_review::SplitDiffRow::Collapsed(block) => {
                Self::show_collapsed(ui, block, params, code_width, pending_toggles);
            }
        }
    }

    fn show_line(
        ui: &mut egui::Ui,
        line: &crate::diff_review::SplitDiffLine,
        side: SplitSide,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        crate::widgets::AlignCenter::new()
            .content(|ui_row| {
                let (cell, opposite) = split_cells(line, side);
                show_split_line_side(ui_row, cell, opposite, code_width, palette);
            })
            .show(ui);
    }

    fn show_collapsed(
        ui: &mut egui::Ui,
        block: &crate::diff_review::UnchangedBlock,
        params: &SplitColumnParams<'_>,
        code_width: f32,
        pending_toggles: &mut Vec<egui::Id>,
    ) {
        let expanded = DiffViewerSplitStateOps::is_block_expanded(params.ctx, params.file, block);
        let block_id = DiffViewerSplitStateOps::block_id(params.file, block);
        if !expanded {
            if DiffViewerRowOps::show_collapsed_side(
                ui,
                block.line_count,
                code_width,
                params.palette,
                false,
            ) {
                pending_toggles.push(block_id);
            }
            return;
        }

        if show_expanded_split_lines(ui, &block.lines, params.side, code_width, params.palette) {
            pending_toggles.push(block_id);
        }
    }
}

pub(super) struct SplitColumnParams<'a> {
    pub(super) area_id: &'static str,
    pub(super) ctx: &'a egui::Context,
    pub(super) file: &'a crate::diff_review::DiffReviewFile,
    pub(super) palette: &'a DiffViewerPalette,
    pub(super) previous_offset: f32,
    pub(super) side: SplitSide,
}

fn show_expanded_split_lines(
    ui: &mut egui::Ui,
    lines: &[crate::diff_review::DiffLine],
    side: SplitSide,
    code_width: f32,
    palette: &DiffViewerPalette,
) -> bool {
    let mut toggle_clicked = false;
    for (index, line) in lines.iter().enumerate() {
        let cell = split_cell(line, side);
        crate::widgets::AlignCenter::new()
            .content(|ui_row| {
                if index == 0 {
                    toggle_clicked |=
                        show_expanded_split_line(ui_row, cell.as_ref(), code_width, palette);
                } else {
                    DiffViewerRowOps::show_split_cell(ui_row, cell.as_ref(), code_width, palette);
                }
            })
            .show(ui);
    }
    toggle_clicked
}

fn show_expanded_split_line(
    ui: &mut egui::Ui,
    cell: Option<&crate::diff_review::DiffCell>,
    code_width: f32,
    palette: &DiffViewerPalette,
) -> bool {
    let tone = cell
        .map(|it| DiffViewerRowOps::tone_for(it.kind))
        .unwrap_or(super::style::DiffTone::Normal);
    let line_number = cell.map(|it| it.line_number);
    let toggle_clicked = DiffViewerRowOps::line_number_toggle_cell(ui, line_number, palette);
    let text = cell.map(|it| it.text.as_str()).unwrap_or_default();
    let ranges = cell
        .map(|it| it.highlight_ranges.as_slice())
        .unwrap_or_default();
    DiffViewerRowOps::code_cell(ui, text, code_width, tone, palette, ranges);
    toggle_clicked
}

fn show_split_line_side(
    ui: &mut egui::Ui,
    cell: Option<&crate::diff_review::DiffCell>,
    opposite_cell: Option<&crate::diff_review::DiffCell>,
    code_width: f32,
    palette: &DiffViewerPalette,
) {
    if let Some(cell) = cell {
        DiffViewerRowOps::show_split_cell(ui, Some(cell), code_width, palette);
        return;
    }

    if let Some(opposite) = opposite_cell {
        DiffViewerRowOps::show_split_placeholder(ui, opposite, code_width, palette);
        return;
    }

    DiffViewerRowOps::show_split_cell(ui, None, code_width, palette);
}

fn split_cells(
    line: &crate::diff_review::SplitDiffLine,
    side: SplitSide,
) -> (
    Option<&crate::diff_review::DiffCell>,
    Option<&crate::diff_review::DiffCell>,
) {
    match side {
        SplitSide::Before => (line.before.as_ref(), line.after.as_ref()),
        SplitSide::After => (line.after.as_ref(), line.before.as_ref()),
    }
}

fn split_cell(
    line: &crate::diff_review::DiffLine,
    side: SplitSide,
) -> Option<crate::diff_review::DiffCell> {
    let line_number = match side {
        SplitSide::Before => line.before_line_number,
        SplitSide::After => line.after_line_number,
    };
    line_number.map(|number| crate::diff_review::DiffCell {
        line_number: number,
        text: line.text.clone(),
        kind: line.kind,
        highlight_ranges: line.highlight_ranges.clone(),
    })
}
