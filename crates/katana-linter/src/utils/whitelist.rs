use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WhitelistEntry {
    pub file: String,
    pub line_content: String,
}

#[derive(Debug, Deserialize)]
pub struct WhitelistConfig {
    pub exemptions: HashMap<String, Vec<WhitelistEntry>>,
}

pub struct WhitelistOps;

static WHITELIST: OnceLock<Option<WhitelistConfig>> = OnceLock::new();

impl WhitelistOps {
    fn load_whitelist() -> Option<WhitelistConfig> {
        let root = super::file_collector::LinterFileOps::workspace_root().ok()?;
        let path = root.join("ast-linter-whitelist.json");
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn is_whitelisted(file: &Path, line_content: &str, rule: &str) -> bool {
        let config = WHITELIST.get_or_init(Self::load_whitelist);
        let Some(config) = config else {
            return false;
        };

        if let Some(entries) = config.exemptions.get(rule) {
            let file_str = file.to_string_lossy();
            for entry in entries {
                // WHY: Support relative paths from workspace root in the whitelist.
                if file_str.ends_with(&entry.file) && line_content.trim() == entry.line_content.trim() {
                    return true;
                }
            }
        }

        false
    }
}
