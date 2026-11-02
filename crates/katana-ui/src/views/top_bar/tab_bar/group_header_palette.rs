use super::group_header_popup::GroupHeaderPopup;
use eframe::egui;

const PALETTE_SIZE: f32 = 16.0;
const PALETTE_RADIUS: f32 = 8.0;
const PALETTE_STROKE: f32 = 2.0;
const GROUP_COLORS: [&str; 7] = [
    "#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C",
];

impl GroupHeaderPopup<'_> {
    pub(super) fn render_palette(&mut self, ui: &mut egui::Ui, current: &mut String) {
        let mut selected = None;
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| selected = Self::render_palette_chips(ui, current))
            .show(ui);
        if let Some(new_color) = selected {
            *current = new_color;
        }
    }

    fn render_palette_chips(ui: &mut egui::Ui, current: &str) -> Option<String> {
        GROUP_COLORS
            .into_iter()
            .find(|color| Self::render_palette_chip(ui, color, current))
            .map(str::to_owned)
    }

    fn render_palette_chip(ui: &mut egui::Ui, color: &str, current: &str) -> bool {
        let color32 = egui::Color32::from_hex(color).unwrap_or_default();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(PALETTE_SIZE, PALETTE_SIZE), egui::Sense::click());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, color));
        ui.painter()
            .circle_filled(rect.center(), PALETTE_RADIUS, color32);
        if current == color {
            ui.painter().circle_stroke(
                rect.center(),
                PALETTE_RADIUS,
                egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()),
            );
        }
        response.clicked()
    }
}
