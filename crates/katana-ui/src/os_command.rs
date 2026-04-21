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

    /* WHY: Renders a string with `{{os_cmd:KEY}}` using the ShortcutWidget for keys */
    pub fn render_mixed(ui: &mut eframe::egui::Ui, text: &str) {
        ui.horizontal_wrapped(|ui| {
            /* WHY: Very simple parser since we only expect basic substitutions */
            let mut remaining = text;
            let json = include_str!("../resources/os_commands.json");
            let dictionary: std::collections::HashMap<String, OsCommand> =
                serde_json::from_str(json).unwrap_or_default();
            while let Some(start_idx) = remaining.find("{{os_cmd:") {
                let prefix = &remaining[..start_idx];
                if !prefix.is_empty() {
                    ui.label(prefix);
                }

                const OS_CMD_PREFIX_LEN: usize = "{{os_cmd:".len();
                let after_prefix = &remaining[start_idx + OS_CMD_PREFIX_LEN..];
                if let Some(end_idx) = after_prefix.find("}}") {
                    let key = &after_prefix[..end_idx];
                    let raw_shortcut = if let Some(cmd) = dictionary.get(key) {
                        cmd.display().to_string()
                    } else {
                        key.to_string()
                    };
                    crate::widgets::ShortcutWidget::new(&raw_shortcut).ui(ui);
                    remaining = &after_prefix[end_idx + 2..];
                } else {
                    const OS_CMD_PREFIX: &str = "{{os_cmd:";
                    ui.label(OS_CMD_PREFIX);
                    remaining = after_prefix;
                }
            }
            if !remaining.is_empty() {
                ui.label(remaining);
            }
        });
    }

    /* WHY: Renders a shortcut string in a fixed-width row layout, with the label on the left
    and the shortcut badge right-aligned, mirroring the appearance in the Shortcut Settings panel. */
    pub fn render_shortcut_row(ui: &mut egui::Ui, text: &str) {
        let regex = regex::Regex::new(r"\{\{os_cmd:([^}]+)\}\}").unwrap();

        let mut key = String::new();
        let mut label_text = text.to_string();

        if let Some(cap) = regex.captures(text) {
            key = cap.get(1).unwrap().as_str().to_string();
            let matched = cap.get(0).unwrap().as_str();
            label_text = label_text.replace(matched, "");
            /* WHY: Clean up leading colons or hyphens and whitespace */
            label_text = label_text
                .trim_start_matches(|c: char| c == ':' || c == '-' || c.is_whitespace())
                .to_string();
            label_text = label_text
                .trim_end_matches(|c: char| c == ':' || c == '-' || c.is_whitespace())
                .to_string();
        }

        let raw_shortcut = if !key.is_empty() {
            let json = include_str!("../resources/os_commands.json");
            let dictionary: std::collections::HashMap<String, OsCommand> =
                serde_json::from_str(json).unwrap_or_default();
            if let Some(cmd) = dictionary.get(&key) {
                cmd.display().to_string()
            } else {
                key.clone()
            }
        } else {
            String::new()
        };

        const MIN_ROW_WIDTH: f32 = 200.0;
        let available_w = ui.available_width().max(MIN_ROW_WIDTH);

        ui.allocate_ui_with_layout(
            eframe::egui::vec2(available_w, 0.0), // 0.0 means natural height shrink wrap
            eframe::egui::Layout::left_to_right(eframe::egui::Align::Center),
            |ui| {
                ui.label(label_text);
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| {
                        if !raw_shortcut.is_empty() {
                            crate::widgets::ShortcutWidget::new(&raw_shortcut).ui(ui);
                        }
                    },
                );
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_command_ops_get() {
        /* WHY: verify it doesn't return the raw key for a known command */
        let result = OsCommandOps::get("save_document");
        assert_ne!(result, "save_document");

        /* WHY: Unkown key returns the key itself */
        let missing = OsCommandOps::get("missing_key_123");
        assert_eq!(missing, "missing_key_123");
    }
}
