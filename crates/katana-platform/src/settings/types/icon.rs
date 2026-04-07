use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/* WHY: Defines overrides for specific icons. */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IconOverride {
    /* WHY: The vendor to retrieve the icon from (e.g. "feather", "heroicons_solid") */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /* WHY: A specific hex color overriding any vendor default */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_hex: Option<String>,
}

/* WHY: A preset bundles a collection of overrides. */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IconPreset {
    pub name: String,
    #[serde(default)]
    pub overrides: HashMap<String, IconOverride>, // Keyed by icon logic name, e.g., "ui/settings"
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
                color_hex: Some("#ff0000".to_string()),
            },
        );

        let mut preset_overrides = HashMap::new();
        preset_overrides.insert(
            "files/document".to_string(),
            IconOverride {
                vendor: Some("heroicons".to_string()),
                color_hex: None,
            },
        );

        let settings = IconSettings {
            custom_presets: vec![IconPreset {
                name: "My Preset".to_string(),
                overrides: preset_overrides,
            }],
            active_preset: Some("My Preset".to_string()),
            active_overrides: overrides,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: IconSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
        assert_eq!(
            deserialized.get_override("ui/settings").unwrap().color_hex,
            Some("#ff0000".to_string())
        );
    }
}
