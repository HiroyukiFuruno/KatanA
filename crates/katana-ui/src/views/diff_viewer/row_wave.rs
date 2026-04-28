use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;
use katana_platform::theme::Rgba;

const REMOVED_WAVE_ALPHA: u8 = 115;
const REMOVED_WAVE_AMPLITUDE: f32 = 1.5;
const REMOVED_WAVE_LENGTH: f32 = 7.0;
const REMOVED_WAVE_STEP_X: f32 = 2.0;
const REMOVED_WAVE_ROW_GAP: f32 = 5.0;

pub(super) struct DiffViewerWaveOps;

impl DiffViewerWaveOps {
    pub(super) fn paint(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        if !matches!(tone, DiffTone::Removed | DiffTone::Added) {
            return;
        }

        let base_color = palette.text_for(tone);
        paint(ui, rect, base_color);
    }

    pub(super) fn paint_removed(ui: &mut egui::Ui, rect: egui::Rect, palette: &DiffViewerPalette) {
        let base_color = palette.text_for(DiffTone::Removed);
        paint(ui, rect, base_color);
    }
}

fn paint(ui: &mut egui::Ui, rect: egui::Rect, base_color: egui::Color32) {
    let stroke = egui::Stroke::new(1.0, color_with_alpha(base_color, REMOVED_WAVE_ALPHA));
    let mut y = rect.top() + REMOVED_WAVE_AMPLITUDE + 1.0;

    while y < rect.bottom() {
        paint_wave_row(ui.painter(), rect, y, stroke);
        y += REMOVED_WAVE_ROW_GAP;
    }
}

fn paint_wave_row(painter: &egui::Painter, rect: egui::Rect, y: f32, stroke: egui::Stroke) {
    let mut points = Vec::new();
    let mut x = rect.left();
    while x <= rect.right() {
        let phase = ((x - rect.left()) / REMOVED_WAVE_LENGTH) * std::f32::consts::TAU;
        points.push(egui::pos2(x, y + phase.sin() * REMOVED_WAVE_AMPLITUDE));
        x += REMOVED_WAVE_STEP_X;
    }
    painter.add(egui::Shape::line(points, stroke));
}

fn color_with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(Rgba {
        r: color.r(),
        g: color.g(),
        b: color.b(),
        a: alpha,
    })
}
