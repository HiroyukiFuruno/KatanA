pub mod dark;
pub mod light;
#[cfg(test)]
mod tests;
pub mod types;

use std::sync::atomic::Ordering;
pub use types::*;

impl DiagramColorPreset {
    pub const DEFAULT_EDITOR_FONT_SIZE: f32 = 14.0;

    pub fn dark() -> &'static Self {
        dark::DarkOps::get()
    }

    pub fn light() -> &'static Self {
        light::LightOps::get()
    }

    pub fn is_dark_mode() -> bool {
        DARK_MODE.load(Ordering::Relaxed)
    }

    pub fn set_dark_mode(is_dark: bool) {
        DARK_MODE.store(is_dark, Ordering::Relaxed);
    }

    pub fn current() -> &'static Self {
        if Self::is_dark_mode() {
            Self::dark()
        } else {
            Self::light()
        }
    }

    pub fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
        const HEX_RGB_LEN: usize = 6;
        const HEX_RADIX: u32 = 16;
        const R_END: usize = 2;
        const G_START: usize = 2;
        const G_END: usize = 4;
        const B_START: usize = 4;

        let hex = hex.strip_prefix('#')?;
        if hex.len() != HEX_RGB_LEN {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..R_END], HEX_RADIX).ok()?;
        let g = u8::from_str_radix(&hex[G_START..G_END], HEX_RADIX).ok()?;
        let b = u8::from_str_radix(&hex[B_START..HEX_RGB_LEN], HEX_RADIX).ok()?;
        Some((r, g, b))
    }

    pub fn relative_luminance(hex: &str) -> Option<f64> {
        const CHANNEL_MAX: f64 = 255.0;
        const LUMA_R: f64 = 0.2126;
        const LUMA_G: f64 = 0.7152;
        const LUMA_B: f64 = 0.0722;

        let (r, g, b) = Self::parse_hex_rgb(hex)?;
        let rf = f64::from(r) / CHANNEL_MAX;
        let gf = f64::from(g) / CHANNEL_MAX;
        let bf = f64::from(b) / CHANNEL_MAX;
        Some(LUMA_R * rf + LUMA_G * gf + LUMA_B * bf)
    }

    pub fn default_proportional_fonts() -> Vec<&'static str> {
        vec![
            "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/AquaKana.ttc",
            "C:/Windows/Fonts/YuGothR.ttc",
            "C:/Windows/Fonts/yugothic.ttf",
            "C:/Windows/Fonts/meiryo.ttc",
            "C:/Windows/Fonts/segoeui.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]
    }

    pub fn default_monospace_fonts() -> Vec<&'static str> {
        vec![
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/SFMono-Regular.otf",
            "/System/Library/Fonts/Monaco.ttf",
            "C:/Windows/Fonts/consola.ttf",
            "C:/Windows/Fonts/cour.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        ]
    }

    pub fn default_emoji_fonts() -> Vec<&'static str> {
        vec![
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            "C:/Windows/Fonts/seguiemj.ttf",
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
        ]
    }
}
