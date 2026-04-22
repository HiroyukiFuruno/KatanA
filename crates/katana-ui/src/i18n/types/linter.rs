use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterTranslations {
    pub rule_toggle: String,
    pub docs: String,
    pub disable_rule: String,
    pub disable_rule_desc: String,
}

impl Default for LinterTranslations {
    fn default() -> Self {
        Self {
            rule_toggle: "Toggle Linter Rule: {rule_code} ({rule_name})".to_string(),
            docs: "Docs".to_string(),
            disable_rule: "Disable".to_string(),
            disable_rule_desc: "Disable this rule".to_string(),
        }
    }
}
