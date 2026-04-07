use crate::Violation;
use crate::utils::LinterFileOps;
use std::fs;
use std::path::Path;

pub struct IconsSyncOps;

const EXTENSION_LEN: usize = 4;

impl IconsSyncOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        let types_rs_path = workspace_root.join("crates/katana-ui/src/icon/types.rs");
        let icons_dir = workspace_root.join("assets/icons/katana");

        if !types_rs_path.exists() || !icons_dir.exists() {
            return violations;
        }

        let content = match fs::read_to_string(&types_rs_path) {
            Ok(c) => c,
            Err(_) => return violations,
        };

        let mut registered_icons = std::collections::HashSet::new();
        for line in content.lines() {
            Self::extract_registered_icon(line, &mut registered_icons);
        }

        let svg_files = LinterFileOps::collect_files_by_extension(&icons_dir, "svg");
        let mut actual_icons = std::collections::HashSet::new();

        for svg_path in svg_files {
            let Ok(rel_path) = svg_path.strip_prefix(&icons_dir) else {
                continue;
            };

            let mut path_str = rel_path.to_string_lossy().to_string();
            if path_str.ends_with(".svg") {
                path_str.truncate(path_str.len() - EXTENSION_LEN);
            }
            /* WHY: Standardize path separators for cross-platform robustness */
            let normalized_path = path_str.replace('\\', "/");
            actual_icons.insert(normalized_path.clone());

            if !registered_icons.contains(&normalized_path) {
                violations.push(Violation {
                    file: svg_path.clone(),
                    line: 1,
                    column: 0,
                    message: format!(
                        "SVG file found but not registered in types.rs `define_icons!` macro: {}",
                        normalized_path
                    ),
                });
            }
        }

        for (i, line) in content.lines().enumerate() {
            Self::check_macro_registration(line, i, &types_rs_path, &actual_icons, &mut violations);
        }

        violations
    }

    fn extract_registered_icon(
        line: &str,
        registered_icons: &mut std::collections::HashSet<String>,
    ) {
        let Some(pos) = line.find("=>") else {
            return;
        };
        let Some(start) = line[pos..].find('"') else {
            return;
        };
        let Some(end) = line[pos + start + 1..].find('"') else {
            return;
        };
        let icon_path = &line[pos + start + 1..pos + start + 1 + end];
        registered_icons.insert(icon_path.to_string());
    }

    fn check_macro_registration(
        line: &str,
        i: usize,
        types_rs_path: &Path,
        actual_icons: &std::collections::HashSet<String>,
        violations: &mut Vec<Violation>,
    ) {
        let Some(pos) = line.find("=>") else {
            return;
        };
        let Some(start) = line[pos..].find('"') else {
            return;
        };
        let Some(end) = line[pos + start + 1..].find('"') else {
            return;
        };
        let icon_path = &line[pos + start + 1..pos + start + 1 + end];
        if !actual_icons.contains(icon_path) {
            violations.push(Violation {
                file: types_rs_path.to_path_buf(),
                line: i + 1,
                column: pos + start,
                message: format!("Icon registered in macro but SVG file does not exist in assets/icons/katana: {}.svg", icon_path),
            });
        }
    }
}
