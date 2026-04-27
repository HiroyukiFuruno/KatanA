use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::preset_state::PresetState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Ignore,
    #[default]
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LinterPreset {
    pub name: String,
    #[serde(default)]
    pub rule_severity: HashMap<String, RuleSeverity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinterSettings {
    #[serde(default = "default_linter_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub use_workspace_local_config: bool,
    #[serde(default)]
    pub rule_severity: HashMap<String, RuleSeverity>,
    #[serde(default)]
    pub custom_presets: Vec<LinterPreset>,
    #[serde(default = "default_linter_preset_state")]
    pub preset_state: PresetState,
}

fn default_linter_enabled() -> bool {
    true
}

fn default_linter_preset_state() -> PresetState {
    PresetState::built_in("katana", "KatanA")
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            use_workspace_local_config: false,
            rule_severity: HashMap::new(),
            custom_presets: Vec::new(),
            preset_state: default_linter_preset_state(),
        }
    }
}

impl LinterSettings {
    pub(crate) fn normalize(&mut self) {
        if self.preset_state.current.is_none() {
            self.preset_state = default_linter_preset_state();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linter_custom_presets_roundtrip() {
        let mut rule_severity = HashMap::new();
        rule_severity.insert("MD013".to_string(), RuleSeverity::Error);

        let settings = LinterSettings {
            rule_severity,
            custom_presets: vec![LinterPreset {
                name: "Strict local".to_string(),
                rule_severity: HashMap::from([("MD048".to_string(), RuleSeverity::Ignore)]),
            }],
            preset_state: PresetState::user("Strict local"),
            ..Default::default()
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: LinterSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
    }

    #[test]
    fn linter_missing_preset_state_defaults_to_katana() {
        let settings: LinterSettings =
            serde_json::from_value(serde_json::json!({ "enabled": true }))
                .expect("linter settings must deserialize");

        let current = settings
            .preset_state
            .current
            .expect("current preset must exist");
        assert_eq!(current.id, "katana");
        assert_eq!(current.label, "KatanA");
        assert!(!settings.preset_state.modified);
    }

    #[test]
    fn linter_normalize_restores_empty_preset_state_to_katana() {
        let mut settings: LinterSettings =
            serde_json::from_value(serde_json::json!({ "preset_state": {} }))
                .expect("linter settings must deserialize");

        settings.normalize();

        let current = settings
            .preset_state
            .current
            .expect("current preset must exist");
        assert_eq!(current.id, "katana");
        assert_eq!(current.label, "KatanA");
    }
}
