use super::row::{CELL_MARGIN_X, CELL_MARGIN_Y, ROW_HEIGHT};
use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

const AVG_MONOSPACE_GLYPH_WIDTH: f32 = 7.5;
const TEXT_EXTRA_PADDING: f32 = 12.0;

pub(super) struct DiffViewerRowLabelOps;

impl DiffViewerRowLabelOps {
    pub(super) fn fixed(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        egui::Frame::NONE
            .fill(palette.gutter_background)
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                ui.set_min_width(width);
                let rich = egui::RichText::new(text)
                    .monospace()
                    .color(label_color(tone, palette));
                ui.allocate_ui_with_layout(
                    egui::vec2(width, ROW_HEIGHT),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.add(egui::Label::new(rich));
                    },
                );
            });
    }

    pub(super) fn clickable(
        ui: &mut egui::Ui,
        text: &str,
        width: f32,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) -> egui::Response {
        let cell_width = width.max(text_display_width(text));
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| clickable_label(ui, text, cell_width, tone, palette))
            .inner
    }
}

fn clickable_label(
    ui: &mut egui::Ui,
    text: &str,
    cell_width: f32,
    tone: DiffTone,
    palette: &DiffViewerPalette,
) -> egui::Response {
    ui.allocate_ui_with_layout(
        egui::vec2(cell_width, ROW_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui_row| {
            egui::Frame::NONE
                .fill(palette.background_for(tone))
                .inner_margin(egui::Margin::symmetric(0, 0))
                .show(ui_row, |ui_bg| {
                    ui_bg.set_min_width(cell_width);
                    ui_bg.set_max_width(cell_width);
                    let rich = egui::RichText::new(text)
                        .monospace()
                        .color(palette.text_for(tone));
                    ui_bg.add(egui::Label::new(rich).sense(egui::Sense::click()))
                })
                .inner
        },
    )
    .inner
}

fn text_display_width(text: &str) -> f32 {
    (text.chars().count() as f32 * AVG_MONOSPACE_GLYPH_WIDTH) + TEXT_EXTRA_PADDING
}

fn label_color(tone: DiffTone, palette: &DiffViewerPalette) -> egui::Color32 {
    if matches!(tone, DiffTone::Normal | DiffTone::Collapsed) {
        palette.secondary_text
    } else {
        palette.text_for(tone)
    }
}
