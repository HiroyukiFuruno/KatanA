use crate::Violation;
use std::collections::HashMap;
use std::path::Path;

pub struct ShortcutOps;

impl ShortcutOps {
    pub fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let os_command_path = root.join("crates/katana-ui/resources/os_commands.json");

        let Ok(content) = std::fs::read_to_string(&os_command_path) else {
            return violations;
        };

        let parsed =
            serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::Value::Null);
        let Some(obj) = parsed.as_object() else {
            return violations;
        };

        let mut seen_shortcuts: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (key, value) in obj {
            if key.starts_with("modifier_") {
                continue;
            }

            if let Some(os_obj) = value.as_object() {
                Self::process_os_shortcuts(
                    key,
                    os_obj,
                    &content,
                    &os_command_path,
                    &mut seen_shortcuts,
                    &mut violations,
                );
            }
        }

        violations.sort_by_key(|v| v.line);
        violations
    }

    fn process_os_shortcuts(
        key: &str,
        os_obj: &serde_json::Map<String, serde_json::Value>,
        content: &str,
        os_command_path: &Path,
        seen_shortcuts: &mut HashMap<String, HashMap<String, String>>,
        violations: &mut Vec<Violation>,
    ) {
        for (os, shortcut_val) in os_obj {
            let Some(shortcut) = shortcut_val.as_str() else {
                continue;
            };

            if shortcut.trim().is_empty() {
                continue;
            }

            let os_map = seen_shortcuts.entry(os.to_string()).or_default();

            if let Some(existing_key) = os_map.get(shortcut) {
                /* WHY: Find line number roughly */
                let line_num = content
                    .lines()
                    .enumerate()
                    .find(|(_, line)| {
                        line.contains(shortcut) && line.contains(&format!("\"{os}\""))
                    })
                    .map(|(idx, _)| idx + 1)
                    .unwrap_or(1);

                violations.push(Violation {
                    file: os_command_path.to_path_buf(),
                    line: line_num,
                    column: 0,
                    message: format!(
                        "Duplicate shortcut '{}' found for OS '{}'. Used by both '{}' and '{}'.",
                        shortcut, os, existing_key, key
                    ),
                });
            } else {
                os_map.insert(shortcut.to_string(), key.to_string());
            }
        }
    }
}
