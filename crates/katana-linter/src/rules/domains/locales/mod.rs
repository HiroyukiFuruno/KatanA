use crate::JsonNodeKind;
use crate::Violation;
use crate::utils::LinterJsonOps;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub mod audit;
pub mod catalog;
pub mod discovery;
pub mod structure;
pub mod values;

use catalog::LocaleCatalogOps;
use discovery::LocaleDiscoveryOps;
use structure::LocaleStructureOps;
use values::LocaleValueOps;

pub use audit::LocaleAuditOps as LocaleAudit;

pub struct LocaleOps;

impl LocaleOps {
    pub fn lint(locale_dir: &Path) -> Vec<Violation> {
        let locale_files = match Self::get_locale_files_or_error(locale_dir) {
            Ok(files) => files,
            Err(v) => return v,
        };

        let language_codes = match LocaleCatalogOps::parse_languages_catalog(locale_dir) {
            Ok(codes) => codes,
            Err(violations) => return violations,
        };
        let mut all_violations =
            LocaleCatalogOps::compare_languages_catalog(locale_dir, &locale_files, &language_codes);

        let ja_path = locale_dir.join("ja.json");
        let en_path = locale_dir.join("en.json");
        let Some((baseline_shape, baseline_placeholders, en_values)) =
            Self::load_locale_baseline(&ja_path, &en_path, &mut all_violations)
        else {
            return all_violations;
        };

        for file in locale_files {
            Self::process_single_locale_file(
                &file,
                &baseline_shape,
                &baseline_placeholders,
                &en_values,
                &mut all_violations,
            );
        }

        all_violations
    }

    fn get_locale_files_or_error(
        locale_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, Vec<Violation>> {
        let locale_files = LocaleDiscoveryOps::collect_locale_json_files(locale_dir);
        if locale_files.is_empty() {
            return Err(vec![crate::utils::ViolationReporterOps::locale_violation(
                locale_dir,
                format!(
                    "No locale JSON files found for analysis: {}",
                    locale_dir.display()
                ),
            )]);
        }
        Ok(locale_files)
    }

    #[allow(clippy::type_complexity)]
    fn load_locale_baseline(
        ja_path: &Path,
        en_path: &Path,
        all_violations: &mut Vec<Violation>,
    ) -> Option<(
        BTreeMap<String, JsonNodeKind>,
        BTreeMap<String, BTreeSet<String>>,
        BTreeMap<String, String>,
    )> {
        match LocaleStructureOps::build_locale_baseline(ja_path, en_path) {
            Ok(baseline) => Some(baseline),
            Err(violations) => {
                all_violations.extend(violations);
                None
            }
        }
    }

    fn process_single_locale_file(
        file: &Path,
        baseline_shape: &BTreeMap<String, JsonNodeKind>,
        baseline_placeholders: &BTreeMap<String, BTreeSet<String>>,
        en_values: &BTreeMap<String, String>,
        all_violations: &mut Vec<Violation>,
    ) {
        let is_base_locale = file.ends_with("ja.json") || file.ends_with("en.json");
        if is_base_locale {
            return;
        }

        let value = match LinterJsonOps::parse_json_file(file) {
            Ok(value) => value,
            Err(violations) => {
                all_violations.extend(violations);
                return;
            }
        };

        let mut shape = BTreeMap::new();
        let mut placeholders = BTreeMap::new();
        let mut values = BTreeMap::new();
        LinterJsonOps::collect_json_shape(&value, None, &mut shape);
        LinterJsonOps::collect_json_placeholders(&value, None, &mut placeholders);
        LinterJsonOps::collect_json_values(&value, None, &mut values);

        all_violations.extend(LocaleStructureOps::compare_locale_shape(
            file,
            baseline_shape,
            &shape,
        ));
        all_violations.extend(LocaleStructureOps::compare_locale_placeholders(
            file,
            baseline_shape,
            baseline_placeholders,
            &placeholders,
        ));
        all_violations.extend(LocaleValueOps::compare_locale_values(
            file, en_values, &values,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_locale_files_reports_empty_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let violations = LocaleOps::lint(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("No locale JSON files found for analysis")
        );
    }
}
