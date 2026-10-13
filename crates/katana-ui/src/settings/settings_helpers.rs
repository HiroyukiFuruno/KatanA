use super::types::*;
use crate::app_state::SettingsTab;
use eframe::egui;

pub(super) fn active_tab_title(tab: &SettingsTab) -> String {
    let tabs = &crate::i18n::I18nOps::get().settings.tabs;
    let (key, default) = match tab {
        SettingsTab::Theme => ("theme", "Theme"),
        SettingsTab::Icons => ("icon", "Icons"),
        SettingsTab::Font => ("font", "Font"),
        SettingsTab::Layout => ("layout", "Layout"),
        SettingsTab::Workspace => ("workspace", "Workspace"),
        SettingsTab::Updates => ("updates", "Updates"),
        SettingsTab::Behavior => ("behavior", "Behavior"),
        SettingsTab::Shortcuts => ("shortcuts", "Shortcuts"),
        SettingsTab::Linter => ("linter", "Linter"),
    };
    tabs.iter()
        .find(|t| t.key == key)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| default.to_string())
}

pub(super) fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.label(egui::RichText::new(text).size(SECTION_HEADER_SIZE).strong());
    ui.add_space(SECTION_HEADER_MARGIN);
    ui.separator();
    ui.add_space(SUBSECTION_SPACING);
}

pub(super) fn add_styled_slider<'a>(ui: &mut egui::Ui, slider: egui::Slider<'a>) -> egui::Response {
    let selection_color = ui.visuals().selection.bg_fill;
    let saved_active_bg = ui.visuals().widgets.active.bg_fill;
    let saved_hovered_bg = ui.visuals().widgets.hovered.bg_fill;
    let saved_inactive_bg = ui.visuals().widgets.inactive.bg_fill;

    ui.visuals_mut().widgets.active.bg_fill = selection_color;
    ui.visuals_mut().widgets.hovered.bg_fill = selection_color;
    ui.visuals_mut().widgets.inactive.bg_fill =
        crate::theme_bridge::ThemeBridgeOps::from_rgba_unmultiplied(
            selection_color.r(),
            selection_color.g(),
            selection_color.b(),
            SLIDER_RAIL_OPACITY,
        );

    let border_stroke = egui::Stroke::new(SLIDER_BORDER_WIDTH, selection_color);
    let saved_active_stroke = ui.visuals().widgets.active.bg_stroke;
    let saved_hovered_stroke = ui.visuals().widgets.hovered.bg_stroke;
    let saved_inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.visuals_mut().widgets.active.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = border_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = border_stroke;

    let response = ui.add(slider);

    ui.visuals_mut().widgets.active.bg_fill = saved_active_bg;
    ui.visuals_mut().widgets.hovered.bg_fill = saved_hovered_bg;
    ui.visuals_mut().widgets.inactive.bg_fill = saved_inactive_bg;
    ui.visuals_mut().widgets.active.bg_stroke = saved_active_stroke;
    ui.visuals_mut().widgets.hovered.bg_stroke = saved_hovered_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = saved_inactive_stroke;

    response
}
