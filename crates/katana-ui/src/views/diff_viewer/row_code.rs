use super::row::{CELL_MARGIN_X, CELL_MARGIN_Y, ROW_HEIGHT};
use super::row_text::DiffViewerTextOps;
use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

pub(super) struct DiffViewerCodeCellOps;

impl DiffViewerCodeCellOps {
    pub(super) fn show(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
        highlight_ranges: &[crate::diff_review::TextRange],
    ) {
        let segments = DiffViewerTextOps::segments(text, highlight_ranges, tone);
        let content_width = DiffViewerTextOps::display_width(&segments);
        let cell_width = width.max(content_width);

        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(cell_width, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui_row| {
                        Self::show_text_area(ui_row, &segments, content_width, tone, palette);
                        Self::fill_rest(ui_row, cell_width, content_width);
                    },
                );
            });
    }

    fn show_text_area(
        ui: &mut egui::Ui,
        segments: &[super::row_text::TextSegment],
        content_width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        egui::Frame::NONE
            .fill(palette.background_for(tone))
            .inner_margin(egui::Margin::symmetric(0, 0))
            .show(ui, |ui_bg| {
                ui_bg.set_min_width(content_width);
                ui_bg.set_max_width(content_width);
                ui_bg.spacing_mut().item_spacing.x = 0.0;
                crate::widgets::AlignCenter::new()
                    .content(|ui_segments| {
                        ui_segments.spacing_mut().item_spacing.x = 0.0;
                        for segment in segments {
                            DiffViewerTextOps::show(ui_segments, segment, tone, palette);
                        }
                    })
                    .show(ui_bg);
            });
    }

    fn fill_rest(ui: &mut egui::Ui, cell_width: f32, content_width: f32) {
        let rest = cell_width - content_width;
        if rest > 0.0 {
            ui.add_space(rest);
        }
    }
}
