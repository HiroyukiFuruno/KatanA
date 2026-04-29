use crate::Violation;
use std::path::{Path, PathBuf};

pub struct ReleaseScriptOps;

impl ReleaseScriptOps {
    pub fn lint(root: &Path) -> Vec<Violation> {
        let script_path = root.join("scripts/build/package-windows.sh");
        let content = match std::fs::read_to_string(&script_path) {
            Ok(content) => content,
            Err(error) => {
                return vec![Self::violation(
                    script_path,
                    1,
                    format!("Failed to read Windows package script: {error}"),
                )];
            }
        };

        let mut violations = Vec::new();
        Self::lint_stale_msi_cleanup(&script_path, &content, &mut violations);
        Self::lint_versioned_msi_selection(&script_path, &content, &mut violations);
        Self::lint_unversioned_head_selection(&script_path, &content, &mut violations);
        violations
    }

    fn lint_stale_msi_cleanup(script_path: &Path, content: &str, violations: &mut Vec<Violation>) {
        let cleanup_line = Self::line_number(content, "rm -f target/wix/*.msi");
        let build_line = Self::line_number(content, "cargo wix --package katana-ui --nocapture");

        if cleanup_line
            .zip(build_line)
            .is_none_or(|(cleanup, build)| cleanup >= build)
        {
            violations.push(Self::violation(
                script_path.to_path_buf(),
                build_line.unwrap_or(1),
                "Delete stale target/wix MSI files before running cargo wix.".to_string(),
            ));
        }
    }

    fn lint_versioned_msi_selection(
        script_path: &Path,
        content: &str,
        violations: &mut Vec<Violation>,
    ) {
        if Self::line_number(content, "-name \"*${CURRENT_VERSION}*.msi\"").is_none() {
            violations.push(Self::violation(
                script_path.to_path_buf(),
                1,
                "Select the MSI by CURRENT_VERSION so cached older installers cannot be copied."
                    .to_string(),
            ));
        }
    }

    fn lint_unversioned_head_selection(
        script_path: &Path,
        content: &str,
        violations: &mut Vec<Violation>,
    ) {
        if let Some(line) = Self::line_number(content, "-name '*.msi' -type f | head -n 1") {
            violations.push(Self::violation(
                script_path.to_path_buf(),
                line,
                "Do not copy the first MSI from target/wix; it may be a cached older version."
                    .to_string(),
            ));
        }
    }

    fn line_number(content: &str, needle: &str) -> Option<usize> {
        content
            .lines()
            .position(|line| line.contains(needle))
            .map(|index| index + 1)
    }

    fn violation(file: PathBuf, line: usize, message: String) -> Violation {
        Violation {
            file,
            line,
            column: 1,
            message,
        }
    }
}
