use super::types::Rgba;
use serde::{Deserialize, Serialize};

/* WHY: Colours specific to code blocks and editors. */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeColors {
    pub background: super::types::Rgb,
    pub text: super::types::Rgb,
    pub line_number_text: super::types::Rgb,
    pub line_number_active_text: super::types::Rgb,
    pub current_line_background: Rgba,
    pub hover_line_background: Rgba,
    pub selection: super::types::Rgb,
    #[serde(default = "CodeColors::default_search_match_color")]
    pub search_match: Rgba,
    #[serde(default = "CodeColors::default_search_active_color")]
    pub search_active: Rgba,
}

pub(crate) const DEFAULT_SEARCH_MATCH_R: u8 = 255;
pub(crate) const DEFAULT_SEARCH_MATCH_G: u8 = 200;
pub(crate) const DEFAULT_SEARCH_MATCH_B: u8 = 0;
pub(crate) const DEFAULT_SEARCH_MATCH_A: u8 = 100;
pub(crate) const DEFAULT_SEARCH_ACTIVE_G: u8 = 100;
pub(crate) const DEFAULT_SEARCH_ACTIVE_A: u8 = 150;

impl CodeColors {
    pub(crate) fn default_search_match_color() -> Rgba {
        Rgba {
            r: DEFAULT_SEARCH_MATCH_R,
            g: DEFAULT_SEARCH_MATCH_G,
            b: DEFAULT_SEARCH_MATCH_B,
            a: DEFAULT_SEARCH_MATCH_A,
        }
    }

    pub(crate) fn default_search_active_color() -> Rgba {
        Rgba {
            r: DEFAULT_SEARCH_MATCH_R,
            g: DEFAULT_SEARCH_ACTIVE_G,
            b: DEFAULT_SEARCH_MATCH_B,
            a: DEFAULT_SEARCH_ACTIVE_A,
        }
    }
}

/* WHY: Colours specific to the markdown preview. */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewColors {
    pub background: super::types::Rgb,
    pub text: super::types::Rgb,
    pub warning_text: super::types::Rgb,
    pub border: super::types::Rgb,
    pub selection: super::types::Rgb,
    #[serde(default = "PreviewColors::default_hover_line_background")]
    pub hover_line_background: Rgba,
}

pub(crate) const DEFAULT_PREVIEW_LINE_BACKGROUND_RGB: u8 = 128;
pub(crate) const DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA: u8 = 15;

impl PreviewColors {
    pub(crate) fn default_hover_line_background() -> Rgba {
        Rgba {
            r: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
            g: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
            b: DEFAULT_PREVIEW_LINE_BACKGROUND_RGB,
            a: DEFAULT_PREVIEW_HOVER_LINE_BACKGROUND_ALPHA,
        }
    }
}
