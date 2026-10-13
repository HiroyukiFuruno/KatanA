/* WHY: Isolated theme and color extraction logic for markdown rendering to maintain modularity and satisfy file length limits. */

use eframe::egui;

pub struct ThemeColorsSet {
    pub text: Option<egui::Color32>,
    pub hover_bg: Option<egui::Color32>,
    pub active_bg: Option<egui::Color32>,
    pub border: Option<egui::Color32>,
    pub selection: Option<egui::Color32>,
}

pub struct MarkdownThemeOps;

impl MarkdownThemeOps {
    pub fn extract_theme_colors(ui: &egui::Ui) -> ThemeColorsSet {
        let theme_colors = ui.ctx().data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        });
        let text_color = theme_colors
            .as_ref()
            .map(|tc| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.text));
        let hover_bg_color = theme_colors.as_ref().map(|tc| {
            crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(tc.preview.hover_line_background)
        });
        let active_bg_color = theme_colors.as_ref().map(|tc| {
            crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(tc.preview.active_line_background)
        });
        let border_color = theme_colors
            .as_ref()
            .map(|tc| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.border));
        let selection_color = theme_colors
            .as_ref()
            .map(|tc| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.selection));

        ThemeColorsSet {
            text: text_color,
            hover_bg: hover_bg_color,
            active_bg: active_bg_color,
            border: border_color,
            selection: selection_color,
        }
    }
}
