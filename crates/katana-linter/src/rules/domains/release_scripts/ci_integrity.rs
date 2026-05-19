use crate::Violation;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub(super) struct ReleaseCiIntegrityOps;

impl ReleaseCiIntegrityOps {
    const CI_BYPASS_MARKERS: [&'static str; 2] = ["[skip ci]", "[ci skip]"];
    const BUMP_COMMIT_MESSAGE: &'static str = "chore: Release v${TARGET_VERSION}";
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

        if Self::line_number(&content, Self::BUMP_COMMIT_MESSAGE).is_none() {
            violations.push(Self::violation(
                script_path,
                1,
                format!(
                    "Release bump commits must use `{}` without CI bypass markers.",
                    Self::BUMP_COMMIT_MESSAGE
                ),
            ));
        }
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
mod tests {
    use crate::rules::ReleaseScriptOps;
    use std::path::Path;

    #[test]
    fn detects_ci_bypass_markers_in_release_automation() {
        let temp_dir = tempfile::tempdir().expect("Test requirement");
        write_file(
            temp_dir.path().join("scripts/release/bump-version.sh"),
            "git commit -m 'chore: Release v${TARGET_VERSION}'\n# [skip ci]\n",
        );
        write_file(
            temp_dir.path().join(".github/workflows/release.yml"),
            "message: chore: release [ci skip]\n",
        );

        let violations = ReleaseScriptOps::lint_ci_integrity(temp_dir.path());

        assert_eq!(violations.len(), 2);
        assert!(
            violations
                .iter()
                .all(|it| { it.message.contains("release-ci-integrity") })
        );
    }

    #[test]
    fn accepts_release_bump_commit_message_without_bypass_marker() {
        let temp_dir = tempfile::tempdir().expect("Test requirement");
        write_file(
            temp_dir.path().join("scripts/release/bump-version.sh"),
            "git commit -S -n -m \"chore: Release v${TARGET_VERSION}\"\n",
        );

        let violations = ReleaseScriptOps::lint_ci_integrity(temp_dir.path());

        assert!(violations.is_empty());
    }

    #[test]
    fn rejects_release_bump_commit_message_without_expected_form() {
        let temp_dir = tempfile::tempdir().expect("Test requirement");
        write_file(
            temp_dir.path().join("scripts/release/bump-version.sh"),
            "git commit -S -n -m \"release ${TARGET_VERSION}\"\n",
        );

        let violations = ReleaseScriptOps::lint_ci_integrity(temp_dir.path());

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("chore: Release v${TARGET_VERSION}")
        );
    }

    fn write_file(path: impl AsRef<Path>, content: &str) {
        let path = path.as_ref();
        let parent = path.parent().expect("Test requirement");
        std::fs::create_dir_all(parent).expect("Test requirement");
        std::fs::write(path, content).expect("Test requirement");
    }
}
