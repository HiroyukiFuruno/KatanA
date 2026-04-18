use crate::Violation;
use std::path::Path;

const MAX_SOURCE_LINES: usize = 200;
const MAX_TEST_LINES: usize = 300;

pub struct FileLengthOps;

impl FileLengthOps {
    pub fn lint(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
        let mut violations = Vec::new();

        let path_str = path.to_string_lossy().replace('\\', "/");
        let is_test = path_str.contains("/tests/") || path_str.ends_with("tests.rs");

        let max_allowed = if is_test {
            MAX_TEST_LINES
        } else {
            MAX_SOURCE_LINES
        };

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

        if lines > max_allowed {
            violations.push(Violation {
                file: path.to_path_buf(),
                line: 1,
                column: 1,
                message: format!(
                    "File exceeds {max_allowed}-line limit (current: {lines}, excluding tests). Split into smaller modules."
                ),
            });
        }

        violations
    }
}
