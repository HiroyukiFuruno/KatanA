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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterSettings {
    #[serde(default = "default_linter_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub use_workspace_local_config: bool,
    #[serde(default)]
    pub rule_severity: HashMap<String, RuleSeverity>,
    #[serde(default)]
    pub preset_state: PresetState,
}

fn default_linter_enabled() -> bool {
    true
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            use_workspace_local_config: false,
            rule_severity: HashMap::new(),
            preset_state: PresetState::built_in("katana", "KatanA"),
        }
    }
}
