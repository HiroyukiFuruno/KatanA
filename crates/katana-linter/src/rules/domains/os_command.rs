use crate::Violation;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::Path;

const OS_CMD_PREFIX_LEN: usize = 9;
const OS_CMD_PLACEHOLDER: &str = "{{os_cmd:";
/// WHY: Unicode representation of the command key to avoid triggering the linter itself.
const CMD_ICON: char = '\u{2318}';

const BANNED_MODIFIERS: &[&str] = &["Cmd+", "Ctrl+", "Option+", "Alt+"];

pub struct OsCommandOps;

impl OsCommandOps {
    pub fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let (tx, rx) = std::sync::mpsc::channel();

        /* WHY: Gather all keys from os_commands.json */
        let os_command_path = root.join("crates/katana-ui/resources/os_commands.json");
        let valid_keys = Self::load_valid_keys(&os_command_path);

        /* WHY: Scan *.rs and *.md in crates/ */
        let crates_dir = root.join("crates");
        let walker = WalkBuilder::new(crates_dir)
            .types(
                ignore::types::TypesBuilder::new()
                    .add_defaults()
                    .select("rust")
                    .select("markdown")
                    .select("json")
                    .build()
                    .unwrap(),
            )
            .build_parallel();

        walker.run(|| {
            let tx = tx.clone();
            let valid_keys = valid_keys.clone();
            Box::new(move |result| {
                let Ok(entry) = result else {
                    return ignore::WalkState::Continue;
                };
                Self::process_file(&entry, &valid_keys, &tx);
                ignore::WalkState::Continue
            })
        });

        drop(tx);
        for v in rx {
            violations.push(v);
        }

        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
        violations
    }

    fn load_valid_keys(path: &Path) -> HashSet<String> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return HashSet::new();
        };
        let parsed =
            serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::Value::Null);
        if let Some(obj) = parsed.as_object() {
            return obj.keys().cloned().collect();
        }
        HashSet::new()
    }

    fn process_file(
        entry: &ignore::DirEntry,
        valid_keys: &HashSet<String>,
        tx: &std::sync::mpsc::Sender<Violation>,
    ) {
        let path = entry.path();
        if !path.is_file()
            || path.ends_with("os_commands.json")
            || path.ends_with("rules/domains/os_command.rs")
        {
            return;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };

        for (line_idx, line) in content.lines().enumerate() {
            if line.contains(CMD_ICON) {
                let _ = tx.send(Violation {
                    file: path.to_path_buf(),
                    line: line_idx + 1,
                    column: 0,
                    message: "Hardcoded shortcut cmd icon found. Use os_command.json standard mechanisms.".to_string(),
                });
            }

            for banned in BANNED_MODIFIERS {
                if line.contains(banned) {
                    let _ = tx.send(Violation {
                        file: path.to_path_buf(),
                        line: line_idx + 1,
                        column: 0,
                        message: format!("Hardcoded shortcut modifier '{}' found. Use os_commands.json standard mechanisms.", banned),
                    });
                }
            }

            if path
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "json")
            {
                Self::check_placeholders(line, valid_keys, path, line_idx, tx);
            }
        }
    }

    fn check_placeholders(
        line: &str,
        valid_keys: &HashSet<String>,
        path: &Path,
        line_idx: usize,
        tx: &std::sync::mpsc::Sender<Violation>,
    ) {
        let mut start_idx = 0;
        while let Some(start) = line[start_idx..].find(OS_CMD_PLACEHOLDER) {
            let absolute_start = start_idx + start;
            if let Some(end) = line[absolute_start..].find("}}") {
                let key = &line[absolute_start + OS_CMD_PREFIX_LEN..absolute_start + end];
                if !valid_keys.contains(key) {
                    let _ = tx.send(Violation {
                        file: path.to_path_buf(),
                        line: line_idx + 1,
                        column: 0,
                        message: format!("Invalid os_cmd key '{}'", key),
                    });
                }
                start_idx = absolute_start + end + 2;
            } else {
                break;
            }
        }
    }
}
