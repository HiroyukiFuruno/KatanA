use eframe::egui::Color32;
use katana_core::canvas::CanvasColor;

use crate::theme_bridge::ThemeBridgeOps;

const COLOR_RADIX: u32 = 16;
const PRESET_COLORS: [&str; 6] = [
    "#D45B5B", "#D68C42", "#B9A43D", "#58A16A", "#459FAE", "#9A70C3",
];

pub(super) struct CanvasColors;

impl CanvasColors {
    pub(super) fn resolve(color: Option<&CanvasColor>, default: Color32) -> Color32 {
        let value = match color {
            Some(CanvasColor::Hex(value)) => Some(value.as_str()),
            Some(CanvasColor::Preset(index)) => index
                .checked_sub(1)
                .and_then(|index| PRESET_COLORS.get(usize::from(index)))
                .copied(),
            None => None,
        };
        value.and_then(Self::hex).unwrap_or(default)
    }

    fn hex(value: &str) -> Option<Color32> {
        let number = u32::from_str_radix(value.strip_prefix('#')?, COLOR_RADIX).ok()?;
        let [_, red, green, blue] = number.to_be_bytes();
        Some(ThemeBridgeOps::from_rgb(red, green, blue))
    }
}
