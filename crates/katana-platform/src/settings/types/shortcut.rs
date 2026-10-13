use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/* WHY: OS-specific personalized shortcut bindings mapping command keys to key chord strings. */
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ShortcutSettings {
    #[serde(default)]
    pub macos: HashMap<String, String>,
    #[serde(default)]
    pub windows: HashMap<String, String>,
    #[serde(default)]
    pub linux: HashMap<String, String>,
}

impl ShortcutSettings {
    pub fn current_os_bindings(&self) -> &HashMap<String, String> {
        let os = std::env::consts::OS;
        match os {
            "macos" => &self.macos,
            "windows" => &self.windows,
            _ => &self.linux,
        }
    }
}
