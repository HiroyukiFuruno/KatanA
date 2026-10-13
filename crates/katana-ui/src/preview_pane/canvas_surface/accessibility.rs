use eframe::egui;

pub(in super::super) struct CanvasLabels;

impl CanvasLabels {
    pub(in super::super) fn register_ui(
        ui: &egui::Ui,
        id: egui::Id,
        rect: egui::Rect,
        content: &str,
    ) {
        /* WHY: 読み取り内容だけを登録し、Scene のドラッグ操作は横取りしない。 */
        let response = ui.interact(rect, id, egui::Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), content)
        });
    }
}
