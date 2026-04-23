use std::path::Path;

pub struct LocaleCompletenessOps;

impl LocaleCompletenessOps {
    /// Rule: ast-linter-i18n-rule-descriptions-completeness
    ///
    /// Ensures that all official markdownlint rules (except Hidden ones)
    /// have a description translation in each locale file.
    pub fn lint_rule_descriptions_completeness(
        _workspace_root: &Path,
        locale_dir: &Path,
    ) -> Vec<crate::Violation> {
        use crate::rules::markdown::RuleParityStatus;
        use crate::rules::markdown::eval::MarkdownLinterOps;
        use crate::utils::LinterJsonOps;

        let mut violations = Vec::new();
        let registry = MarkdownLinterOps::get_user_configurable_rules();
        let locales = [
            "ja", "en", "ko", "de", "es", "fr", "it", "pt", "zh-CN", "zh-TW",
        ];

        for locale in locales {
            let path = locale_dir.join(format!("{}.json", locale));
            let Ok(json) = LinterJsonOps::parse_json_file(&path) else {
                continue;
            };

            let Some(linter_obj) = json.get("linter").and_then(|l| l.as_object()) else {
                continue;
            };

            let rule_descriptions = match linter_obj
                .get("rule_descriptions")
                .and_then(|r| r.as_object())
            {
                Some(r) => r,
                None => {
                    violations.push(crate::utils::ViolationReporterOps::locale_violation(
                        &path,
                        format!(
                            "Missing `rule_descriptions` object in `linter` section for locale {}",
                            locale
                        ),
                    ));
                    continue;
                }
            };

            for rule in &registry {
                let Some(meta) = rule.official_meta() else {
                    continue;
                };
                if meta.parity == RuleParityStatus::Hidden {
                    continue;
                }

                let code_lower = meta.code.to_lowercase();
                if !rule_descriptions.contains_key(&code_lower) {
                    violations.push(crate::utils::ViolationReporterOps::locale_violation(
                        &path,
                        format!(
                            "Missing translation for markdownlint rule `{}` in locale {}.json",
                            meta.code, locale
                        ),
                    ));
                }
            }
        }

        violations
    }
}
