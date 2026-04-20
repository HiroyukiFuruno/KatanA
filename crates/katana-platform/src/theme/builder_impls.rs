use super::builder::ThemePresetBuilder;
use super::palettes::*;
use crate::theme::colors_code::{CodeColors, PreviewColors};
use crate::theme::types::{Rgb, Rgba, SystemColors, ThemeOps};

impl ThemePresetBuilder {
    const DEFAULT_SEARCH_MATCH_R: u8 = 255;
    const DEFAULT_SEARCH_MATCH_G: u8 = 200;
    const DEFAULT_SEARCH_MATCH_B: u8 = 0;
    const DEFAULT_SEARCH_MATCH_A: u8 = 100;
    const DEFAULT_SEARCH_ACTIVE_G: u8 = 100;
    const DEFAULT_SEARCH_ACTIVE_A: u8 = 150;

    #[allow(clippy::too_many_arguments)]
    pub(super) const fn build_system(
        &self,
        p_bg: Rgb,
        t_sec: Rgb,
        success: Rgb,
        warning: Rgb,
        error: Rgb,
        border: Rgb,
        selection: Rgb,
    ) -> SystemColors {
        SystemColors {
            background: self.background,
            panel_background: p_bg,
            text: self.text,
            text_secondary: t_sec,
            success_text: success,
            warning_text: warning,
            error_text: error,
            accent: self.accent,
            title_bar_text: self.text,
            file_tree_text: t_sec,
            active_file_highlight: ThemeOps::to_rgba(
                self.accent,
                DEFAULT_ACTIVE_FILE_HIGHLIGHT_ALPHA,
            ),
            button_background: ThemeOps::to_rgba(p_bg, DEFAULT_BUTTON_BACKGROUND_ALPHA),
            button_active_background: ThemeOps::to_rgba(self.accent, DEFAULT_BUTTON_ACTIVE_ALPHA),
            border,
            selection,
        }
    }

    pub(super) const fn build_code(
        &self,
        c_bg: Rgb,
        t_sec: Rgb,
        selection: Rgb,
        is_dark: bool,
    ) -> CodeColors {
        let current_line_background = Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: if is_dark {
                DEFAULT_CODE_CURRENT_LINE_DARK_ALPHA
            } else {
                DEFAULT_CODE_CURRENT_LINE_LIGHT_ALPHA
            },
        };
        CodeColors {
            background: c_bg,
            text: self.text,
            line_number_text: t_sec,
            line_number_active_text: self.text,
            current_line_background,
            hover_line_background: ThemeOps::to_rgba(
                self.accent,
                DEFAULT_HOVER_LINE_HIGHLIGHT_ALPHA,
            ),
            selection,
            search_match: Rgba {
                r: Self::DEFAULT_SEARCH_MATCH_R,
                g: Self::DEFAULT_SEARCH_MATCH_G,
                b: Self::DEFAULT_SEARCH_MATCH_B,
                a: Self::DEFAULT_SEARCH_MATCH_A,
            },
            search_active: Rgba {
                r: Self::DEFAULT_SEARCH_MATCH_R,
                g: Self::DEFAULT_SEARCH_ACTIVE_G,
                b: Self::DEFAULT_SEARCH_MATCH_B,
                a: Self::DEFAULT_SEARCH_ACTIVE_A,
            },
        }
    }

    pub(super) const fn build_preview(
        &self,
        warning: Rgb,
        border: Rgb,
        selection: Rgb,
    ) -> PreviewColors {
        PreviewColors {
            background: self.background,
            text: self.text,
            warning_text: warning,
            border,
            selection,
            hover_line_background: ThemeOps::to_rgba(
                self.accent,
                DEFAULT_HOVER_LINE_HIGHLIGHT_ALPHA,
            ),
            active_line_background: ThemeOps::to_rgba(
                self.accent,
                DEFAULT_ACTIVE_LINE_HIGHLIGHT_ALPHA,
            ),
        }
    }
}
