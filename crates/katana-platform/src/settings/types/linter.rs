use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinterSettings {
    #[serde(default)]
    pub disabled_rules: HashSet<String>,
}
