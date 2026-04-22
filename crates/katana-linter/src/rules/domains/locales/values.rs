use crate::Violation;
use crate::utils::{LinterParserOps, ViolationReporterOps};
use std::collections::BTreeMap;
use std::path::Path;

struct LocaleException {
    key: &'static str,
    value: &'static str,
}

/* WHY: Exclusion list for translation value validation.
1. Add identical values across languages (proper nouns, versions, etc.).
2. Use `*` for broad matching.
3. Meaningful words ("File", "Search") MUST NOT be excluded. */
const LOCALE_VALUE_EXCEPTIONS: &[LocaleException] = &[
    LocaleException {
        key: "rust",
        value: "Rust",
    },
    LocaleException {
        key: "support",
        value: "Support",
    },
    LocaleException {
        key: "action_close",
        value: "OK",
    },
    LocaleException {
        key: "key",
        value: "*",
    },
    LocaleException {
        key: "kind",
        value: "*",
    },
    LocaleException {
        key: "*",
        value: "KatanA",
    },
    LocaleException {
        key: "*",
        value: "PlantUML",
    },
    LocaleException {
        key: "*",
        value: "wkhtmltopdf",
    },
    LocaleException {
        key: "*",
        value: "Rust",
    },
    LocaleException {
        key: "render_error",
        value: "*",
    },
    LocaleException {
        key: "*",
        value: "Copyright",
    },
    LocaleException {
        key: "*",
        value: "Runtime",
    },
    LocaleException {
        key: "*",
        value: "Build",
    },
    LocaleException {
        key: "*",
        value: "Version",
    },
    LocaleException {
        key: "*",
        value: "v1.0.0",
    },
    LocaleException {
        key: "*",
        value: "Version: {version}",
    },
    LocaleException {
        key: "*",
        value: "File",
    },
    LocaleException {
        key: "*",
        value: "Sponsor",
    },
    LocaleException {
        key: "*",
        value: "Layout",
    },
    LocaleException {
        key: "*",
        value: "Code",
    },
    LocaleException {
        key: "*",
        value: "Links",
    },
    LocaleException {
        key: "*",
        value: "Theme",
    },
    LocaleException {
        key: "*",
        value: "Architecture",
    },
    LocaleException {
        key: "*",
        value: "Documentation",
    },
    LocaleException {
        key: "*",
        value: "Text",
    },
    LocaleException {
        key: "section_editor",
        value: "Editor",
    },
    LocaleException {
        key: "section_general",
        value: "General",
    },
    LocaleException {
        key: "severity_error",
        value: "Error",
    },
    LocaleException {
        key: "severity_warning",
        value: "Warning",
    },
    LocaleException {
        key: "severity_ignore",
        value: "Ignore",
    },
    LocaleException {
        key: "*",
        value: "Linter",
    },
];

pub struct LocaleValueOps;

impl LocaleValueOps {
    pub fn compare_locale_values(
        file: &Path,
        en_values: &BTreeMap<String, String>,
        actual_values: &BTreeMap<String, String>,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (path, en_val) in en_values {
            let Some(actual_val) = actual_values.get(path) else {
                continue;
            };

            /* WHY: Anti-Lazy Compliance: Reject pseudo-translations like "[TODO-ko] Settings" */
            if actual_val.contains("[TODO") || actual_val.contains("[todo") {
                violations.push(ViolationReporterOps::locale_violation(
                    file,
                    format!(
                        "Locale value at `{path}` contains a pseudo-translation placeholder (\"{actual_val}\"). Lazy cheat detected. Please provide a true native translation."
                    ),
                ));
                continue;
            }

            if actual_val == en_val && !Self::is_allowed_duplicate(path, actual_val) {
                violations.push(ViolationReporterOps::locale_violation(
                    file,
                    format!(
                        "Locale value at `{path}` is identical to English baseline (\"{actual_val}\"). Please translate it."
                    ),
                ));
            }
        }

        violations
    }

    pub fn is_allowed_duplicate(path: &str, value: &str) -> bool {
        if LinterParserOps::is_allowed_string(value) {
            return true;
        }

        LOCALE_VALUE_EXCEPTIONS.iter().any(|ex| {
            let key_matches = ex.key == "*"
                || ex.key == path
                || path.split('.').next_back().is_some_and(|k| k == ex.key);
            key_matches && (ex.value == "*" || ex.value == value)
        })
    }
}
