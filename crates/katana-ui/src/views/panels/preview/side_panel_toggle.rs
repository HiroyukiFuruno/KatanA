use super::side_panels::{LIGHT_MODE_ICON_BG, PreviewSidePanels};
use eframe::egui;

const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 230;
const TOGGLE_BUTTON_SIZE: f32 = 28.0;
const TOGGLE_BUTTON_ROUNDING: u8 = 4;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_toggle_button(
        &mut self,
        ui: &mut egui::Ui,
        icon: crate::Icon,
        is_active: bool,
        tooltip: &str,
        shortcut: Option<&str>,
    ) -> egui::Response {
        #[rustfmt::skip]
        let icon_bg = if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG) };
        #[rustfmt::skip]
        let active_bg = if ui.visuals().dark_mode { ui.visuals().selection.bg_fill } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_ACTIVE_BG) };
        let resp = ui.add(
            egui::Button::image(icon.ui_image(ui, crate::icon::IconSize::Medium))
                .fill(if is_active { active_bg } else { icon_bg })
                .min_size(egui::vec2(TOGGLE_BUTTON_SIZE, TOGGLE_BUTTON_SIZE))
                .corner_radius(egui::CornerRadius::same(TOGGLE_BUTTON_ROUNDING)),
        );

        decorate_toggle_response(ui, resp, tooltip, shortcut)
    }
}

fn decorate_toggle_response(
    ui: &egui::Ui,
    response: egui::Response,
    tooltip: &str,
    shortcut: Option<&str>,
) -> egui::Response {
    let mut text = tooltip.to_string();
    let response = if let Some(shortcut) = shortcut {
        text.push_str(&format!(" ({shortcut})"));
        response.on_hover_ui(|ui| {
            ui.allocate_ui_with_layout(
                egui::Vec2::ZERO,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(tooltip);
                    crate::widgets::ShortcutWidget::new(shortcut).ui(ui);
                },
            );
        })
    } else {
        response.on_hover_text(tooltip)
    };

    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), text.clone())
    });
    response
}
