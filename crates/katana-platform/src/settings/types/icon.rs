use crate::theme::Rgba;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/* WHY: Defines overrides for specific icons. */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IconOverride {
    /* WHY: The vendor to retrieve the icon from (e.g. "feather", "heroicons_solid") */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /* WHY: A specific color overriding any vendor default */
    #[serde(
        alias = "color_hex",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_icon_color"
    )]
    pub color: Option<Rgba>,
    /* WHY: A specific color for the icon button frame/border */
    #[serde(
        alias = "frame_color_hex",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_icon_color"
    )]
    pub frame_color: Option<Rgba>,
}

fn deserialize_icon_color<'de, D>(deserializer: D) -> Result<Option<Rgba>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IconColor {
        Rgba(Rgba),
        Hex(String),
    }

    let opt: Option<IconColor> = Option::deserialize(deserializer)?;
    match opt {
        Some(IconColor::Rgba(rgba)) => Ok(Some(rgba)),
        Some(IconColor::Hex(hex)) => {
            /* WHY: Migrate hex string to Rgba if valid. */
            if let Some(rgba) = hex_to_rgba(&hex) {
                Ok(Some(rgba))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

const HEX_RADIX: u32 = 16;
const DEFAULT_ALPHA: u8 = 255;
const RGB_HEX_LEN: usize = 6;
const RGBA_HEX_LEN: usize = 8;

const RED_START: usize = 0;
const RED_END: usize = 2;
const GREEN_START: usize = 2;
const GREEN_END: usize = 4;
const BLUE_START: usize = 4;
const BLUE_END: usize = 6;
const ALPHA_START: usize = 6;
const ALPHA_END: usize = 8;

/* WHY: Helper to parse various hex string formats (#RRGGBB, #RRGGBBAA) into Rgba. */
fn hex_to_rgba(hex: &str) -> Option<Rgba> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == RGB_HEX_LEN {
        let r = u8::from_str_radix(&hex[RED_START..RED_END], HEX_RADIX).ok()?;
        let g = u8::from_str_radix(&hex[GREEN_START..GREEN_END], HEX_RADIX).ok()?;
        let b = u8::from_str_radix(&hex[BLUE_START..BLUE_END], HEX_RADIX).ok()?;
        Some(Rgba {
            r,
            g,
            b,
            a: DEFAULT_ALPHA,
        })
    } else if hex.len() == RGBA_HEX_LEN {
        let r = u8::from_str_radix(&hex[RED_START..RED_END], HEX_RADIX).ok()?;
        let g = u8::from_str_radix(&hex[GREEN_START..GREEN_END], HEX_RADIX).ok()?;
        let b = u8::from_str_radix(&hex[BLUE_START..BLUE_END], HEX_RADIX).ok()?;
        let a = u8::from_str_radix(&hex[ALPHA_START..ALPHA_END], HEX_RADIX).ok()?;
        Some(Rgba { r, g, b, a })
    } else {
        None
    }
}

/* WHY: A preset bundles a collection of overrides. */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IconPreset {
    /* WHY: Unique name for the preset. */
    pub name: String,
    /* WHY: The overrides defined in this preset, keyed by icon logical name (e.g. "ui/settings"). */
    #[serde(default)]
    pub overrides: HashMap<String, IconOverride>,
}

/* WHY: Global settings for Icon configuration. */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IconSettings {
    /* WHY: List of saved user presets. */
    #[serde(default)]
    pub custom_presets: Vec<IconPreset>,
    /* WHY: The active preset name (if playing a custom preset). None means default behavior. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_preset: Option<String>,
    /* WHY: Free-floating overrides in the active non-preset state. */
    #[serde(default)]
    pub active_overrides: HashMap<String, IconOverride>,
    /* WHY: Whether non-Katana vendor icons should be automatically tinted with their vendor default color. */
    #[serde(default = "default_colorful_vendor_icons")]
    pub colorful_vendor_icons: bool,
}

fn default_colorful_vendor_icons() -> bool {
    false
}

impl IconSettings {
    pub fn get_override(&self, icon_name: &str) -> Option<&IconOverride> {
        self.active_overrides.get(icon_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_settings_serialization() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "ui/settings".to_string(),
            IconOverride {
                vendor: Some("feather".to_string()),
                color: Some(Rgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                frame_color: Some(Rgba {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                }),
            },
        );

        let mut preset_overrides = HashMap::new();
        preset_overrides.insert(
            "files/document".to_string(),
            IconOverride {
                vendor: Some("heroicons".to_string()),
                color: None,
                frame_color: None,
            },
        );

        let settings = IconSettings {
            custom_presets: vec![IconPreset {
                name: "My Preset".to_string(),
                overrides: preset_overrides,
            }],
            active_preset: Some("My Preset".to_string()),
            active_overrides: overrides,
            colorful_vendor_icons: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: IconSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
        assert_eq!(
            deserialized.get_override("ui/settings").unwrap().color,
            Some(Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            })
        );
    }

    #[test]
    fn test_icon_override_hex_migration() {
        let json = r##"{
            "vendor": "feather",
            "color_hex": "#FF000080"
        }"##;
        let ov: IconOverride = serde_json::from_str(json).unwrap();
        assert_eq!(ov.vendor, Some("feather".to_string()));
        assert_eq!(
            ov.color,
            Some(Rgba {
                r: 255,
                g: 0,
                b: 0,
                a: 128
            })
        );
    }
}
