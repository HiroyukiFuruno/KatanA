use crate::Violation;
use crate::utils::LinterFileOps;
use std::fs;
use std::path::Path;

fn check_line_for_invalid_colors(
    path: &Path,
    line_num: usize,
    line_content: &str,
) -> Option<Violation> {
    let attributes = ["fill=\"", "stroke=\""];

    for attr in attributes {
        let mut search_pos = 0;
        while let Some(start) = line_content[search_pos..].find(attr) {
            let start_idx = search_pos + start + attr.len();
            let Some(end) = line_content[start_idx..].find('"') else {
                break;
            };
            let color_val = &line_content[start_idx..start_idx + end].to_lowercase();

            if !is_allowed_svg_color(color_val) {
                return Some(Violation {
                    file: path.to_path_buf(),
                    line: line_num,
                    column: start_idx,
                    message: format!(
                        "Invalid SVG color detected: `{}` in attribute `{}`. All icons must use `#FFFFFF` (or `none`) to support dynamic tinting.",
                        color_val, attr
                    ),
                });
            }
            search_pos = start_idx + end;
        }
    }

    None
}

fn is_allowed_svg_color(color: &str) -> bool {
    matches!(
        color,
        "none" | "white" | "#ffffff" | "#fff" | "currentcolor"
    )
}

fn get_render_policy(_pack_dir: &str) -> &'static str {
    /* WHY: To support new packs that are NativeColor, return "NativeColor" here. */
    /* WHY: Future extension for e.g. "colorful-emojis" => "NativeColor" */
    "TintedMonochrome"
}

/* WHY: Centralises skip logic for tint validation:
 * - NativeColor packs are exempt by design. */
fn should_skip_tint_check(path: &std::path::Path) -> bool {
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect();
    let mut after_icons = components.iter().skip_while(|c| *c != "icons");
    let pack_dir = after_icons
        .next()
        .and_then(|_| after_icons.next())
        .map(|s| s.as_ref())
        .unwrap_or("");
    get_render_policy(pack_dir) == "NativeColor"
}

pub struct SvgOps;

impl SvgOps {
    pub fn lint_svg_colors(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let icons_dir = workspace_root.join("assets/icons");
        if !icons_dir.exists() {
            return violations;
        }
        for path in LinterFileOps::collect_files_by_extension(&icons_dir, "svg") {
            if should_skip_tint_check(&path) {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            /* WHY: For TintedMonochrome packs, all icons must use `#FFFFFF` or `currentColor`
             * to support egui's dynamic tinting without destroying icon shapes. */
            if !content.contains("fill=\"") && !content.contains("stroke=\"") {
                violations.push(Violation {
                    file: path.clone(),
                    line: 1,
                    column: 0,
                    message: "Blackout Bug Detected: SVG has neither `fill` nor `stroke`. It will render as black and fail dynamic tinting. Add `fill=\"#FFFFFF\"` or `stroke=\"#FFFFFF\"`.".to_string(),
                });
            }
            for (i, line) in content.lines().enumerate() {
                if let Some(v) = check_line_for_invalid_colors(&path, i + 1, line) {
                    violations.push(v);
                }
            }
        }
        violations
    }

    pub fn lint_duplicate_svgs(workspace_root: &Path) -> Vec<Violation> {
        use super::icons_sync::IconsSyncOps;
        let mut violations = Vec::new();
        let icons_dir = workspace_root.join("assets/icons");

        if !icons_dir.exists() {
            return violations;
        }

        let whitelist = IconsSyncOps::get_duplicate_whitelist();
        let files = LinterFileOps::collect_files_by_extension(&icons_dir, "svg");
        let mut content_map: std::collections::HashMap<(String, String), Vec<std::path::PathBuf>> =
            std::collections::HashMap::new();

        for path in files {
            let pack_dir = path
                .iter()
                .skip_while(|&c| c != "icons")
                .nth(1)
                .and_then(|s| s.to_str())
                .unwrap_or("katana")
                .to_string();

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            /* WHY: Extract inner SVG content (ignoring <svg ...> wrapper to avoid class noise) */
            let inner_content = match extract_inner_svg(&content) {
                Some(inner) => inner,
                /* WHY: fallback to full content if extraction fails */
                None => content.trim().to_string(),
            };

            content_map
                .entry((pack_dir, inner_content))
                .or_default()
                .push(path);
        }

        for ((pack_dir, _), paths) in content_map {
            if paths.len() <= 1 {
                continue;
            }
            /* WHY: If ALL paths match the whitelist, allow it. */
            if paths.iter().all(|p| {
                let p_str = p.to_string_lossy();
                whitelist.iter().any(|&w| p_str.contains(w))
            }) {
                continue;
            }

            let names = paths
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ");

            for path in paths {
                violations.push(Violation {
                    file: path.clone(),
                    line: 1,
                    column: 0,
                    message: format!(
                        "Duplicate SVG found in theme `{pack_dir}`: identical visual content to {names}. Ensure this icon has a distinct purpose or remove the duplicate."
                    ),
                });
            }
        }

        violations
    }
}

fn extract_inner_svg(c: &str) -> Option<String> {
    let s = c.find("<svg")?;
    let e = c.rfind("</svg>")?;
    let b = c[s..].find('>')? + s;
    if b >= e {
        return None;
    }
    Some(c[b + 1..e].trim().to_string())
}
