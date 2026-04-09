use crate::Violation;
use std::path::Path;

const MAX_FILE_LINES: usize = 200;

pub struct FileLengthOps;

impl FileLengthOps {
    pub fn lint(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
        let mut violations = Vec::new();

        let path_str = path.to_string_lossy().replace('\\', "/");
        if path_str.contains("/tests/") || path_str.ends_with("tests.rs") {
            return violations;
        }

        const LEGACY_LONG_FILES: &[&str] = &[
            "crates/katana-core/src/emoji/raster.rs",
            "crates/katana-ui/src/app/workspace/open.rs",
            "crates/katana-ui/src/html_renderer/render_inline.rs",
            "crates/katana-ui/src/preview_pane/fullscreen_impl.rs",
            "crates/katana-ui/src/preview_pane/section_show.rs",
            "crates/katana-ui/src/preview_pane/slideshow.rs",
            "crates/katana-ui/src/settings/tabs/theme.rs",
            "crates/katana-ui/src/settings/tabs/theme_editor.rs",
            "crates/katana-ui/src/shell_ui/shell_ui_frame.rs",
            "crates/katana-ui/src/views/app_frame/tab_toolbar.rs",
            "crates/katana-ui/src/views/modals/search_tabs.rs",
            "crates/katana-ui/src/views/top_bar/tab_bar/tab_item.rs",
        ];

        if LEGACY_LONG_FILES.iter().any(|f| path_str.ends_with(f)) {
            return violations;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            return violations;
        };

        /* WHY: Line count is computed after stripping test modules to avoid penalizing test-heavy files. */
        let mut lines: usize = 0;
        let mut in_test_mod = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[cfg(test)]") {
                in_test_mod = true;
                continue;
            }
            if in_test_mod && (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) {
                /* WHY: Approximation - stops counting at first #[cfg(test)] mod boundary to exclude test code from line limit. */
                break;
            }
            lines += 1;
        }

        if lines > MAX_FILE_LINES {
            violations.push(Violation {
                file: path.to_path_buf(),
                line: 1,
                column: 1,
                message: format!(
                    "File exceeds {MAX_FILE_LINES}-line limit (current: {lines}, excluding tests). Split into smaller modules."
                ),
            });
        }

        violations
    }
}
