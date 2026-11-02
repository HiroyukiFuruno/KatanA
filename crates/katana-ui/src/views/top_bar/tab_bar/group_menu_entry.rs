use crate::state::document::TabGroup;
use eframe::egui;

pub(crate) struct GroupMenuEntry;

const DOT_PADDING: f32 = 12.0;
const DOT_OFFSET: f32 = 9.0;
const DOT_RADIUS: f32 = 4.0;

impl GroupMenuEntry {
    pub(crate) fn show(ui: &mut egui::Ui, group: &TabGroup) -> egui::Response {
        let response = ui
            .scope(|ui| {
                ui.spacing_mut().button_padding.x += DOT_PADDING;
                ui.button(&group.name)
            })
            .inner;
        let color = egui::Color32::from_hex(&group.color_hex)
            .unwrap_or(ui.visuals().widgets.active.bg_fill);
        let center = egui::pos2(response.rect.left() + DOT_OFFSET, response.rect.center().y);
        ui.painter().circle_filled(center, DOT_RADIUS, color);
        response
    }
}
