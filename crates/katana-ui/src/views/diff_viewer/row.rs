use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

const GRID_SPACING_X: f32 = 2.0;
const GRID_GAP_COUNT: f32 = 3.0;
const LINE_NUMBER_WIDTH: f32 = 46.0;
const SIGN_WIDTH: f32 = 24.0;
const ROW_HEIGHT: f32 = 20.0;
const MIN_SPLIT_CODE_WIDTH: f32 = 380.0;
const MIN_INLINE_CODE_WIDTH: f32 = 760.0;
const CELL_MARGIN_X: i8 = 6;
const CELL_MARGIN_Y: i8 = 1;
const TEXT_EXTRA_PADDING: f32 = 12.0;

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
        Self::code_cell(ui, text, code_width, tone, palette);
    }

    pub(super) fn show_collapsed_side(
        ui: &mut egui::Ui,
        line_count: usize,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        Self::collapse_icon_cell(ui, palette);
        Self::collapsed_text_cell(ui, line_count, code_width, palette);
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
        Self::fixed_label(ui, text, SIGN_WIDTH, tone_for(kind), palette);
    }

    pub(super) fn line_number_cell(
        ui: &mut egui::Ui,
        line_number: Option<usize>,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        let text = line_number.map(|it| it.to_string()).unwrap_or_default();
        Self::fixed_label(ui, &text, LINE_NUMBER_WIDTH, tone, palette);
    }

    pub(super) fn code_cell(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        let content_width = Self::text_display_width(text);
        let cell_width = width.max(content_width);

        /* WHY: Draw only the text-area background (not the whole cell width) so
        that highlight regions do not extend beyond the text range. */
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                /* WHY: Force a fixed row height so left/right rows remain aligned. */
                ui.allocate_ui_with_layout(
                    egui::vec2(cell_width, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui_row| {
                        egui::Frame::NONE
                            .fill(palette.background_for(tone))
                            .inner_margin(egui::Margin::symmetric(0, 0))
                            .show(ui_row, |ui_bg| {
                                ui_bg.set_min_width(content_width);
                                ui_bg.set_max_width(content_width);
                                let rich = egui::RichText::new(text)
                                    .monospace()
                                    .color(palette.text_for(tone));
                                ui_bg.add(egui::Label::new(rich).selectable(false));
                            });

                        /* WHY: Fill remaining space to keep consistent cell width. */
                        let rest = cell_width - content_width;
                        if rest > 0.0 {
                            ui_row.add_space(rest);
                        }
                    },
                );
            });
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

    fn collapse_icon_cell(ui: &mut egui::Ui, palette: &DiffViewerPalette) {
        egui::Frame::NONE
            .fill(palette.gutter_background)
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                ui.set_min_width(LINE_NUMBER_WIDTH);
                ui.add_sized(
                    egui::vec2(LINE_NUMBER_WIDTH, ROW_HEIGHT),
                    crate::Icon::ChevronDown.ui_image(ui, crate::icon::IconSize::Small),
                );
            });
    }

    pub(super) fn collapsed_text_cell(
        ui: &mut egui::Ui,
        line_count: usize,
        code_width: f32,
        palette: &DiffViewerPalette,
    ) {
        let count = line_count.to_string();
        let text = crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().diff_review.collapsed_lines,
            &[("count", &count)],
        );
        Self::code_cell(ui, &text, code_width, DiffTone::Collapsed, palette);
    }

    fn fixed_label(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        let color = if matches!(tone, DiffTone::Normal | DiffTone::Collapsed) {
            palette.secondary_text
        } else {
            palette.text_for(tone)
        };
        egui::Frame::NONE
            .fill(palette.gutter_background)
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                ui.set_min_width(width);
                let rich = egui::RichText::new(text).monospace().color(color);
                ui.allocate_ui_with_layout(
                    egui::vec2(width, ROW_HEIGHT),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.add(egui::Label::new(rich));
                    },
                );
            });
    }

    fn text_display_width(text: &str) -> f32 {
        const AVG_MONOSPACE_GLYPH_WIDTH: f32 = 7.5;
        (text.chars().count() as f32 * AVG_MONOSPACE_GLYPH_WIDTH) + TEXT_EXTRA_PADDING
    }
}

fn tone_for(kind: crate::diff_review::DiffLineKind) -> DiffTone {
    match kind {
        crate::diff_review::DiffLineKind::Unchanged => DiffTone::Normal,
        crate::diff_review::DiffLineKind::Removed => DiffTone::Removed,
        crate::diff_review::DiffLineKind::Added => DiffTone::Added,
    }
}
