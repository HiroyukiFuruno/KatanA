use crate::Violation;
use crate::utils::collect_files_by_extension;
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
            let Some(end) = line_content[start_idx..].find('"') else { break; };
            let color_val = &line_content[start_idx..start_idx + end].to_lowercase();

            if !is_allowed_svg_color(color_val) {
                return Some(Violation {
                    file: path.to_path_buf(),
                    line: line_num,
                    column: start_idx,
                    message: format!(
                        "Invalid SVG color detected: `{}`. All icons must use `#FFFFFF` (or `none`) to support dynamic tinting.",
                        color_val
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

pub struct SvgOps;

impl SvgOps {
    pub fn lint_svg_colors(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let icons_dir = workspace_root.join("assets/icons");
    
        if !icons_dir.exists() {
            return violations;
        }
    
        let files = collect_files_by_extension(&icons_dir, "svg");
    
        for path in files {
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };
    
            /* WHY: We want to ensure all icons are pure white (#FFFFFF) to support egui's dynamic tinting.
               Any other hex colors or color names indicate an inconsistent design asset. */
        
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if let Some(violation) = check_line_for_invalid_colors(&path, i + 1, line) {
                    violations.push(violation);
                }
            }
        }
    
        violations
    }
}
