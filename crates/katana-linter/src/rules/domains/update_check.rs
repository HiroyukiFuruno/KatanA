use crate::Violation;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct UpdateCheckOps;

impl UpdateCheckOps {
    const RAW_BINDING: &'static str = "if let Some(err) = &self.state.update.check_error";
    const LOCALIZED_CALL: &'static str = "UpdateErrorMessageOps::localize(err";
    const RAW_DISPLAY_LOOKAHEAD_LINES: usize = 16;

    pub fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        for file in Self::update_modal_files(root) {
            let content = match std::fs::read_to_string(&file) {
                Ok(content) => content,
                Err(error) => {
                    violations.push(Self::violation(
                        file,
                        1,
                        format!("Failed to read update modal file: {error}"),
                    ));
                    continue;
                }
            };
            Self::lint_file(&file, &content, &mut violations);
        }
        violations
    }

    fn lint_file(file: &Path, content: &str, violations: &mut Vec<Violation>) {
        Self::lint_direct_check_error_display(file, content, violations);
        Self::lint_raw_bound_error_display(file, content, violations);
    }

    fn lint_direct_check_error_display(
        file: &Path,
        content: &str,
        violations: &mut Vec<Violation>,
    ) {
        for (index, line) in content.lines().enumerate() {
            if !line.contains("state.update.check_error") {
                continue;
            }
            if line.contains("ui.label") || line.contains("format!") {
                violations.push(Self::violation(
                    file.to_path_buf(),
                    index + 1,
                    Self::message(),
                ));
            }
        }
    }

    fn lint_raw_bound_error_display(file: &Path, content: &str, violations: &mut Vec<Violation>) {
        let lines: Vec<&str> = content.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !line.contains(Self::RAW_BINDING) {
                continue;
            }
            let window = Self::following_window(&lines, index);
            if window
                .iter()
                .any(|line| line.contains(Self::LOCALIZED_CALL))
            {
                continue;
            }
            if window.iter().any(|line| line.trim() == "err,") {
                violations.push(Self::violation(
                    file.to_path_buf(),
                    index + 1,
                    Self::message(),
                ));
            }
        }
    }

    fn following_window<'a>(lines: &'a [&str], index: usize) -> &'a [&'a str] {
        let end = std::cmp::min(index + Self::RAW_DISPLAY_LOOKAHEAD_LINES, lines.len());
        &lines[index..end]
    }

    fn update_modal_files(root: &Path) -> Vec<PathBuf> {
        let target = root.join("crates/katana-ui/src/views/modals/update");
        if !target.exists() {
            return Vec::new();
        }
        let mut files = Vec::new();
        let walker = WalkBuilder::new(target).standard_filters(true).build();
        for entry in walker.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path.to_path_buf());
            }
        }
        files.sort();
        files
    }

    fn message() -> String {
        "Display update check errors through update_check_error_* i18n messages.".to_string()
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
    use super::*;

    #[test]
    fn detects_raw_bound_update_check_error_display() {
        let source = r#"
            if let Some(err) = &self.state.update.check_error {
                let close = phases::show_error(
                    ctx,
                    &msgs.title,
                    err,
                    &msgs.failed_to_check,
                    &msgs.action_close,
                    color,
                );
            }
        "#;

        let mut violations = Vec::new();
        UpdateCheckOps::lint_file(Path::new("mod.rs"), source, &mut violations);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn accepts_localized_update_check_error_display() {
        let source = r#"
            if let Some(err) = &self.state.update.check_error {
                let message = error_message::UpdateErrorMessageOps::localize(err, msgs);
                let close = phases::show_error(
                    ctx,
                    &msgs.title,
                    &message,
                    &msgs.failed_to_check,
                    &msgs.action_close,
                    color,
                );
            }
        "#;

        let mut violations = Vec::new();
        UpdateCheckOps::lint_file(Path::new("mod.rs"), source, &mut violations);

        assert!(violations.is_empty());
    }
}
