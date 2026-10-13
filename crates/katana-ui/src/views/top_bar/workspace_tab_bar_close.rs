use crate::views::top_bar::workspace_tab_bar_detail::WorkspaceTabBarDetail;
use eframe::egui;

impl WorkspaceTabBarDetail {
    pub(super) fn render_close_button(
        ui: &mut egui::Ui,
        path: &str,
        close_rect: egui::Rect,
        visible: bool,
    ) -> egui::Response {
        if visible {
            let icon_rect = egui::Rect::from_center_size(
                close_rect.center(),
                crate::icon::IconSize::Small.to_vec2(),
            );
            ui.put(
                icon_rect,
                crate::Icon::Close.ui_image(ui, crate::icon::IconSize::Small),
            );
        }
        let sense = if visible {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let response = ui
            .interact(
                close_rect,
                egui::Id::new("workspace_tab_close_button").with(path),
                sense,
            )
            .on_hover_text(crate::i18n::I18nOps::get().tab.close.clone());
        if visible {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    crate::i18n::I18nOps::get().tab.close.clone(),
                )
            });
        }
        response
    }
}
