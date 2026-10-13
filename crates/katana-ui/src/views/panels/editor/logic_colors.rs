use super::types::*;
use eframe::egui;

impl EditorLogicOps {
    /// Resolve editor theme colors from the egui context's temporary data.
    pub fn resolve_editor_colors(ui: &egui::Ui) -> EditorColors {
        ui.ctx().data(|d| -> EditorColors {
            if let Some(tc) = d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                "katana_theme_colors",
            )) {
                (
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.code.background),
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.code.text),
                    Some(crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                        tc.code.selection,
                    )),
                    Some(crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                        tc.code.current_line_background,
                    )),
                    Some(crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                        tc.code.hover_line_background,
                    )),
                    Some(crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                        tc.code.line_number_text,
                    )),
                    Some(crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                        tc.code.line_number_active_text,
                    )),
                )
            } else {
                (
                    ui.visuals().extreme_bg_color,
                    ui.visuals().text_color(),
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            }
        })
    }

    /// Compute the current line highlight color, falling back to a semi-transparent overlay.
    pub fn current_line_highlight_color(
        dark_mode: bool,
        themed_color: Option<egui::Color32>,
    ) -> egui::Color32 {
        const HIGHLIGHT_ALPHA: u8 = 15;
        themed_color.unwrap_or_else(|| {
            if dark_mode {
                crate::theme_bridge::ThemeBridgeOps::from_white_alpha(HIGHLIGHT_ALPHA)
            } else {
                crate::theme_bridge::ThemeBridgeOps::from_black_alpha(HIGHLIGHT_ALPHA)
            }
        })
    }

    /// Compute the hover highlight color for preview-linked lines.
    pub fn hover_line_highlight_color(
        dark_mode: bool,
        themed_color: Option<egui::Color32>,
    ) -> egui::Color32 {
        const HOVER_HIGHLIGHT_ALPHA: u8 = 10;
        themed_color.unwrap_or_else(|| {
            if dark_mode {
                crate::theme_bridge::ThemeBridgeOps::from_white_alpha(HOVER_HIGHLIGHT_ALPHA)
            } else {
                crate::theme_bridge::ThemeBridgeOps::from_black_alpha(HOVER_HIGHLIGHT_ALPHA)
            }
        })
    }
}
