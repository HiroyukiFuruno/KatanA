use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub rule_severity: HashMap<String, RuleSeverity>,
}

fn default_linter_enabled() -> bool {
    true
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            rule_severity: HashMap::new(),
        }
    }
}
