use super::row_code::DiffViewerCodeCellOps;
use super::row_label::DiffViewerRowLabelOps;
use super::row_toggle::DiffViewerRowToggleOps;
use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

const GRID_SPACING_X: f32 = 2.0;
const GRID_GAP_COUNT: f32 = 3.0;
pub(super) const LINE_NUMBER_WIDTH: f32 = 46.0;
const SIGN_WIDTH: f32 = 24.0;
pub(super) const ROW_HEIGHT: f32 = 20.0;
const MIN_SPLIT_CODE_WIDTH: f32 = 380.0;
const MIN_INLINE_CODE_WIDTH: f32 = 760.0;
pub(super) const CELL_MARGIN_X: i8 = 6;
pub(super) const CELL_MARGIN_Y: i8 = 1;

pub(super) struct DiffViewerRowOps;

impl DiffViewerRowOps {
    pub(super) fn show_split_cell(
        ui: &mut egui::Ui,
        cell: Option<&crate::diff_review::DiffCell>,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        let tone = cell.map(|it| tone_for(it.kind)).unwrap_or(DiffTone::Normal);
        Self::line_number_cell(ui, cell.map(|it| it.line_number), tone, palette);
        let text = cell.map(|it| it.text.as_str()).unwrap_or_default();
        let ranges = cell
            .map(|it| it.highlight_ranges.as_slice())
            .unwrap_or_default();
        DiffViewerCodeCellOps::show(ui, text, code_width, tone, palette, ranges);
    }

    pub(super) fn show_split_placeholder(
        ui: &mut egui::Ui,
        opposite: &crate::diff_review::DiffCell,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        let tone = tone_for(opposite.kind);
        let placeholder_width =
            DiffViewerCodeCellOps::content_width(&opposite.text, tone, &opposite.highlight_ranges);
        Self::line_number_cell(ui, None, tone, palette);
        DiffViewerCodeCellOps::show_placeholder(ui, code_width, placeholder_width, tone, palette);
    }

    pub(super) fn show_collapsed_side(
        ui: &mut egui::Ui,
        line_count: usize,
        code_width: f32,
        palette: &DiffViewerPalette,
        expanded: bool,
    ) -> bool {
        let icon_clicked = Self::collapsed_toggle_cell(ui, palette, expanded);
        let text_clicked = Self::collapsed_text_cell(ui, line_count, code_width, palette).clicked();
        icon_clicked || text_clicked
    }

    pub(super) fn sign_cell(
        ui: &mut egui::Ui,
        kind: crate::diff_review::DiffLineKind,
        palette: &DiffViewerPalette,
    ) {
        let text = match kind {
            crate::diff_review::DiffLineKind::Removed => "-",
            crate::diff_review::DiffLineKind::Added => "+",
            crate::diff_review::DiffLineKind::Unchanged => " ",
        };
        DiffViewerRowLabelOps::fixed(ui, text, SIGN_WIDTH, tone_for(kind), palette);
    }

    pub(super) fn line_number_cell(
        ui: &mut egui::Ui,
        line_number: Option<usize>,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        let text = line_number.map(|it| it.to_string()).unwrap_or_default();
        DiffViewerRowLabelOps::fixed(ui, &text, LINE_NUMBER_WIDTH, tone, palette);
    }

    pub(super) fn line_number_toggle_cell(
        ui: &mut egui::Ui,
        line_number: Option<usize>,
        palette: &DiffViewerPalette,
    ) -> bool {
        let text = line_number.map(|it| it.to_string()).unwrap_or_default();
        DiffViewerRowToggleOps::show_gutter(ui, palette, true, &text)
    }

    pub(super) fn code_cell(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
        highlight_ranges: &[crate::diff_review::TextRange],
    ) {
        DiffViewerCodeCellOps::show(ui, text, width, tone, palette, highlight_ranges);
    }

    pub(super) fn split_code_width(ui: &egui::Ui) -> f32 {
        let reserved = LINE_NUMBER_WIDTH * 2.0 + GRID_SPACING_X * GRID_GAP_COUNT;
        ((ui.available_width() - reserved) / 2.0).max(MIN_SPLIT_CODE_WIDTH)
    }

    pub(super) fn inline_code_width(ui: &egui::Ui) -> f32 {
        let reserved = LINE_NUMBER_WIDTH + SIGN_WIDTH + GRID_SPACING_X * 2.0;
        (ui.available_width() - reserved).max(MIN_INLINE_CODE_WIDTH)
    }

    pub(super) fn tone_for(kind: crate::diff_review::DiffLineKind) -> DiffTone {
        tone_for(kind)
    }

    pub(super) fn collapsed_toggle_cell(
        ui: &mut egui::Ui,
        palette: &DiffViewerPalette,
        expanded: bool,
    ) -> bool {
        DiffViewerRowToggleOps::show(ui, palette, expanded)
    }

    pub(super) fn collapsed_text_cell(
        ui: &mut egui::Ui,
        line_count: usize,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) -> egui::Response {
        let count = line_count.to_string();
        let text = crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().diff_review.collapsed_lines,
            &[("count", &count)],
        );
        DiffViewerRowLabelOps::clickable(ui, &text, code_width, DiffTone::Collapsed, palette)
    }
}

fn tone_for(kind: crate::diff_review::DiffLineKind) -> DiffTone {
    match kind {
        crate::diff_review::DiffLineKind::Unchanged => DiffTone::Normal,
        crate::diff_review::DiffLineKind::Removed => DiffTone::Removed,
        crate::diff_review::DiffLineKind::Added => DiffTone::Added,
    }
}
