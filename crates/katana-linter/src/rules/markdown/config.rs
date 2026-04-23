use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

/// Manages reading and writing `.markdownlint.json` configuration files.
pub struct MarkdownLintConfig {
    pub raw: Value,
}

impl MarkdownLintConfig {
    /// Loads a config from the given path. If it does not exist or is invalid, returns a default config.
    pub fn load(path: &Path) -> Self {
        let raw = if path.exists() {
            fs::read_to_string(path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_else(|| serde_json::json!({ "default": true }))
        } else {
            serde_json::json!({ "default": true })
        };
        Self { raw }
    }

    /// Saves the current configuration to the given path.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json_str = serde_json::to_string_pretty(&self.raw)?;
        fs::write(path, json_str)
    }

    /// Sets a boolean rule configuration (e.g., "MD001": false)
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) {
        if let Some(obj) = self.raw.as_object_mut() {
            obj.insert(rule_id.to_string(), Value::Bool(enabled));
        }
    }

    /// Sets a property value for a specific rule (e.g., "MD013": { "line_length": 100 })
    pub fn set_rule_property(&mut self, rule_id: &str, prop_key: &str, value: Value) {
        if let Some(obj) = self.raw.as_object_mut() {
            /* WHY: Preserve existing keys but skip the root object itself.
             * The macro does not serialize `self`, it just iterates standard rules. */
            /* WHY: markdownlint allows `true` to mean "use default config". */
            /* WHY: If we are setting a specific property, it MUST be an object. */
            if !obj.contains_key(rule_id) || !obj[rule_id].is_object() {
                obj.insert(rule_id.to_string(), Value::Object(Map::new()));
            }
            if let Some(rule_obj) = obj.get_mut(rule_id).and_then(|v| v.as_object_mut()) {
                rule_obj.insert(prop_key.to_string(), value);
            }
        }
    }

    /// Gets a property value for a specific rule.
    pub fn get_rule_property(&self, rule_id: &str, prop_key: &str) -> Option<&Value> {
        self.raw
            .get(rule_id)
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get(prop_key))
    }
}
