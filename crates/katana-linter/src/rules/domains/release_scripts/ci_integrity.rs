use crate::Violation;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub(super) struct ReleaseCiIntegrityOps;

impl ReleaseCiIntegrityOps {
    const CI_BYPASS_MARKERS: [&'static str; 2] = ["[skip ci]", "[ci skip]"];
    const RELEASE_CI_CAPABILITY: &'static str = "release-ci-integrity";

    pub(super) fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        Self::lint_no_ci_bypass_markers(root, &mut violations);
        Self::lint_bump_commit_message(root, &mut violations);
        violations
    }

    fn lint_no_ci_bypass_markers(root: &Path, violations: &mut Vec<Violation>) {
        for file in Self::release_automation_files(root) {
            let content = match std::fs::read_to_string(&file) {
                Ok(content) => content,
                Err(error) => {
                    violations.push(Self::violation(
                        file,
                        1,
                        format!("Failed to read release automation file: {error}"),
                    ));
                    continue;
                }
            };

            Self::lint_file_for_ci_bypass_markers(&file, &content, violations);
        }
    }

    fn lint_file_for_ci_bypass_markers(
        file: &Path,
        content: &str,
        violations: &mut Vec<Violation>,
    ) {
        for marker in Self::CI_BYPASS_MARKERS {
            if let Some(line) = Self::line_number(content, marker) {
                violations.push(Self::violation(
                    file.to_path_buf(),
                    line,
                    format!(
                        "Do not embed CI bypass marker `{marker}` in release automation; capability `{}` forbids it.",
                        Self::RELEASE_CI_CAPABILITY
                    ),
                ));
            }
        }
    }

    fn lint_bump_commit_message(root: &Path, violations: &mut Vec<Violation>) {
        let script_path = root.join("scripts/release/bump-version.sh");
        let content = match std::fs::read_to_string(&script_path) {
            Ok(content) => content,
            Err(error) => {
                violations.push(Self::violation(
                    script_path,
                    1,
                    format!("Failed to read release bump script: {error}"),
                ));
                return;
            }
        };

        if !Self::has_valid_bump_commit(&content) {
            violations.push(Self::violation(
                script_path,
                1,
                "Release bump must have exactly one signed version and issue-bound commit with the required paths.".to_string(),
            ));
        }
    }

    fn has_valid_bump_commit(content: &str) -> bool {
        let Ok(command) = regex::Regex::new(r"\bgit(?:[ \t]+[^ \t;&|()]+){0,8}[ \t]+commit\b")
        else {
            return false;
        };
        let mut found = 0;
        let joined = content.replace("\\\n", " ");
        for line in joined
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
        {
            for _ in command.find_iter(line) {
                found += 1;
                if !Self::valid_commit_line(line.trim()) {
                    return false;
                }
            }
        }
        found == 1
    }

    fn valid_commit_line(line: &str) -> bool {
        const PREFIX: &str = "git commit -S -m \"chore: v${TARGET_VERSION}";
        const SUFFIX: &str =
            " (#${KATANA_RELEASE_ISSUE})\" -- Cargo.toml Cargo.lock \"$INFO_PLIST\"";
        let Some(message) = line
            .strip_prefix(PREFIX)
            .and_then(|rest| rest.strip_suffix(SUFFIX))
        else {
            return false;
        };
        !message.is_empty()
            && !message.is_ascii()
            && !message.chars().any(|character| {
                matches!(character, '$' | '`' | '\\' | '\"') || character.is_control()
            })
    }

    fn release_automation_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        Self::collect_files(root.join("scripts/release"), &mut files);
        Self::collect_files(root.join(".github/workflows"), &mut files);
        files.sort();
        files
    }

    fn collect_files(root: PathBuf, files: &mut Vec<PathBuf>) {
        if !root.exists() {
            return;
        }

        let walker = WalkBuilder::new(root).standard_filters(true).build();
        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_path_buf());
            }
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

#[cfg(test)]
#[path = "ci_integrity_tests.rs"]
mod tests;
