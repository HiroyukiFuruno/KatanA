use crate::Violation;
use crate::utils::LinterFileOps;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct IconsSyncOps;

const EXTENSION_LEN: usize = 4;

impl IconsSyncOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();

        let types_rs_path = workspace_root.join("crates/katana-ui/src/icon/types.rs");
        let icons_dir = workspace_root.join("assets/icons");

        if !types_rs_path.exists() || !icons_dir.exists() {
            return violations;
        }

        let content = match fs::read_to_string(&types_rs_path) {
            Ok(c) => c,
            Err(_) => return violations,
        };

        /* WHY: 1. Extract registered icons from define_icons! */
        let mut registered_icons = HashSet::new();
        for line in content.lines() {
            Self::extract_registered_icon(line, &mut registered_icons);
        }

        /* WHY: 2. Discover theme packs */
        let mut theme_packs = Vec::new();
        let Ok(entries) = fs::read_dir(&icons_dir) else {
            return violations;
        };
        for entry in entries.flatten() {
            if entry.file_type().unwrap().is_dir() {
                theme_packs.push(entry.file_name().to_string_lossy().to_string());
            }
        }

        /* WHY: We will store actual_icons per theme pack to check sync */
        let mut per_theme_icons: HashMap<String, HashSet<String>> = HashMap::new();
        /* WHY: For duplicates check */
        let mut file_contents: HashMap<String, Vec<String>> = HashMap::new();

        let whitelist = Self::get_duplicate_whitelist();

        for theme in &theme_packs {
            let theme_dir = icons_dir.join(theme);
            let svg_files = LinterFileOps::collect_files_by_extension(&theme_dir, "svg");
            let mut actual_icons = HashSet::new();

            for svg_path in svg_files {
                let Ok(rel_path) = svg_path.strip_prefix(&theme_dir) else {
                    continue;
                };

                let mut path_str = rel_path.to_string_lossy().to_string();
                if path_str.ends_with(".svg") {
                    path_str.truncate(path_str.len() - EXTENSION_LEN);
                }
                let normalized_path = path_str.replace('\\', "/");
                actual_icons.insert(normalized_path.clone());

                /* WHY: Task 5.3: Detect unregistered SVGs */
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

                /* WHY: Collect content for duplicate detection */
                let Ok(content) = fs::read(&svg_path) else {
                    continue;
                };
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let hash = format!("{:x}", hasher.finish());

                file_contents
                    .entry(hash)
                    .or_default()
                    .push(svg_path.to_string_lossy().to_string());
            }
            per_theme_icons.insert(theme.clone(), actual_icons);
        }

        /* WHY: Ensure all registered icons exist in ALL theme packs */
        for (i, line) in content.lines().enumerate() {
            let Some(pos) = line.find("=>") else {
                continue;
            };
            let Some(start) = line[pos..].find('"') else {
                continue;
            };
            let Some(end) = line[pos + start + 1..].find('"') else {
                continue;
            };
            let icon_path = &line[pos + start + 1..pos + start + 1 + end];

            for theme in &theme_packs {
                let Some(actual_icons) = per_theme_icons.get(theme) else {
                    continue;
                };
                if !actual_icons.contains(icon_path) {
                    violations.push(Violation {
                        file: types_rs_path.clone(),
                        line: i + 1,
                        column: pos + start,
                        message: format!(
                            "Icon `{}` is registered in marco, but missing from theme pack `{}`.",
                            icon_path, theme
                        ),
                    });
                }
            }
        }

        /* WHY: Duplicate SVG detection */
        for (_hash, paths) in file_contents {
            if paths.len() <= 1 {
                continue;
            }
            /* WHY: Check if ALL these paths are in the whitelist */
            let all_whitelisted = paths
                .iter()
                .all(|p| whitelist.iter().any(|&w| p.contains(w)));

            if !all_whitelisted {
                let mut message = "Duplicate SVGs detected (identical contents). ".to_string();
                message.push_str(
                    "If intentional, add to the whitelist. Otherwise, unique icons are required. ",
                );
                message.push_str(
                    "See `.gemini/antigravity/skills/svg-icon-management/SKILL.md` for policy. ",
                );
                message.push_str(&format!("Duplicates: {:?}", paths));

                violations.push(Violation {
                    file: Path::new(&paths[0]).to_path_buf(),
                    line: 1,
                    column: 0,
                    message,
                });
            }
        }

        violations
    }

    fn extract_registered_icon(line: &str, registered_icons: &mut HashSet<String>) {
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

    #[rustfmt::skip]
    fn get_duplicate_whitelist() -> &'static [&'static str] {
        /* WHY: Whitelist by relative path substring or exact path */
        &[
            "navigation/chevron_down.svg", "navigation/chevron_left.svg", "navigation/chevron_right.svg",
            "navigation/triangle_down.svg", "navigation/triangle_left.svg", "navigation/triangle_right.svg",
            "view/pan_down.svg", "view/pan_right.svg", "ui/expand_all.svg", "ui/collapse_all.svg",
            "ui/close.svg", "ui/remove.svg", "ui/copy.svg", "system/history.svg", "system/hourglass.svg",
            "system/recent.svg", "system/github.svg", "layout/swap_horizontal.svg",
            "layout/swap_vertical.svg", "files/explorer.svg",
        ]
    }
}
