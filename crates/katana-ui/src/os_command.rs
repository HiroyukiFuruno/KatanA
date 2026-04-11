use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsCommand {
    pub mac: String,
    pub windows: String,
    pub linux: String,
}

impl OsCommand {
    pub fn display(&self) -> &str {
        #[cfg(target_os = "macos")]
        {
            &self.mac
        }
        #[cfg(target_os = "windows")]
        {
            &self.windows
        }
        #[cfg(target_os = "linux")]
        {
            &self.linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            &self.linux
        }
    }
}

pub struct OsCommandOps;

impl OsCommandOps {
    pub fn get(key: &str) -> String {
        let json = include_str!("../resources/os_commands.json");
        let dictionary: std::collections::HashMap<String, OsCommand> =
            serde_json::from_str(json).unwrap_or_default();
        if let Some(cmd) = dictionary.get(key) {
            cmd.display().to_string()
        } else {
            key.to_string()
        }
    }

    pub fn replace_in_text(text: &str) -> String {
        let json = include_str!("../resources/os_commands.json");
        let dictionary: std::collections::HashMap<String, OsCommand> =
            serde_json::from_str(json).unwrap_or_default();
        let mut result = text.to_string();
        for (k, v) in dictionary {
            let placeholder = format!("{{{{os_cmd:{}}}}}", k);
            result = result.replace(&placeholder, v.display());
        }
        result
    }
}
