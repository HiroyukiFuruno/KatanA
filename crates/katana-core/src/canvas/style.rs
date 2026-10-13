use serde::Deserialize;

const HEX_COLOR_LENGTH: usize = 7;
const PRESET_RED: u8 = 1;
const PRESET_ORANGE: u8 = 2;
const PRESET_YELLOW: u8 = 3;
const PRESET_GREEN: u8 = 4;
const PRESET_CYAN: u8 = 5;
const PRESET_PURPLE: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum CanvasColor {
    Preset(u8),
    Hex(String),
}

impl TryFrom<String> for CanvasColor {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "1" => Ok(Self::Preset(PRESET_RED)),
            "2" => Ok(Self::Preset(PRESET_ORANGE)),
            "3" => Ok(Self::Preset(PRESET_YELLOW)),
            "4" => Ok(Self::Preset(PRESET_GREEN)),
            "5" => Ok(Self::Preset(PRESET_CYAN)),
            "6" => Ok(Self::Preset(PRESET_PURPLE)),
            _ if value.len() == HEX_COLOR_LENGTH
                && value.starts_with('#')
                && value.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit()) =>
            {
                Ok(Self::Hex(value))
            }
            _ => Err("invalid canvas color"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeEnd {
    None,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundStyle {
    Cover,
    Ratio,
    Repeat,
}
