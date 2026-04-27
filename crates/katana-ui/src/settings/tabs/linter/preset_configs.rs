use katana_markdown_linter::MarkdownLintConfig;
use katana_platform::settings::RuleSeverity;
use std::collections::HashMap;

pub(super) const KATANA_PRESET_ID: &str = "katana";
pub(super) const DISABLED_PRESET_ID: &str = "disabled";
pub(super) const STRICT_PRESET_ID: &str = "strict";
pub(super) const WARNING_PRESET_ID: &str = "warning";

pub(super) struct LinterPresetData {
    pub(super) config: MarkdownLintConfig,
    pub(super) rule_severity: HashMap<String, RuleSeverity>,
}

pub(super) struct LinterPresetConfigOps;

impl LinterPresetConfigOps {
    pub(super) fn built_in(id: &str) -> Option<LinterPresetData> {
        match id {
            KATANA_PRESET_ID => Some(Self::katana()),
            DISABLED_PRESET_ID => Some(Self::disabled()),
            STRICT_PRESET_ID => Some(Self::official_default(RuleSeverity::Error)),
            WARNING_PRESET_ID => Some(Self::official_default(RuleSeverity::Warning)),
            _ => None,
        }
    }

    fn katana() -> LinterPresetData {
        let config = crate::linter_default_config::MarkdownLinterDefaultConfigOps::load();
        let rule_severity = Self::severity_by_fix_support(&config);
        LinterPresetData {
            config,
            rule_severity,
        }
    }

    fn disabled() -> LinterPresetData {
        let config = MarkdownLintConfig {
            raw: serde_json::json!({ "default": false }),
        };
        let rule_severity = Self::constant_severity(&config, RuleSeverity::Ignore);
        LinterPresetData {
            config,
            rule_severity,
        }
    }

    fn official_default(severity: RuleSeverity) -> LinterPresetData {
        let config = MarkdownLintConfig::default();
        let rule_severity = Self::constant_severity(&config, severity);
        LinterPresetData {
            config,
            rule_severity,
        }
    }

    fn severity_by_fix_support(config: &MarkdownLintConfig) -> HashMap<String, RuleSeverity> {
        let options = config.to_lint_options();
        Self::rule_meta()
            .into_iter()
            .map(|(rule_id, is_fixable)| {
                let severity = if !Self::rule_enabled(&options, &rule_id) {
                    RuleSeverity::Ignore
                } else if is_fixable {
                    RuleSeverity::Error
                } else {
                    RuleSeverity::Warning
                };
                (rule_id, severity)
            })
            .collect()
    }

    fn constant_severity(
        config: &MarkdownLintConfig,
        severity: RuleSeverity,
    ) -> HashMap<String, RuleSeverity> {
        let options = config.to_lint_options();
        Self::rule_meta()
            .into_iter()
            .map(|(rule_id, _)| {
                let severity = if Self::rule_enabled(&options, &rule_id) {
                    severity
                } else {
                    RuleSeverity::Ignore
                };
                (rule_id, severity)
            })
            .collect()
    }

    fn rule_enabled(options: &katana_markdown_linter::LintOptions, rule_id: &str) -> bool {
        options
            .rules
            .get(rule_id)
            .map(|rule_config| rule_config.enabled)
            .unwrap_or(true)
    }

    fn rule_meta() -> Vec<(String, bool)> {
        katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
            .into_iter()
            .filter_map(|rule| {
                rule.official_meta()
                    .map(|meta| (meta.code.to_string(), meta.is_fixable))
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn katana_preset_uses_bundled_config_and_fixable_severity() {
        let preset = LinterPresetConfigOps::built_in(KATANA_PRESET_ID).unwrap();

        assert_eq!(preset.config.raw["MD013"], false);
        assert_eq!(
            preset.rule_severity.get("MD013"),
            Some(&RuleSeverity::Ignore)
        );
        assert_eq!(
            preset.rule_severity.get("MD048"),
            Some(&RuleSeverity::Error)
        );
        assert_eq!(
            preset.rule_severity.get("MD001"),
            Some(&RuleSeverity::Warning)
        );
    }

    #[test]
    fn warning_and_strict_presets_use_official_default_config() {
        let warning = LinterPresetConfigOps::built_in(WARNING_PRESET_ID).unwrap();
        let strict = LinterPresetConfigOps::built_in(STRICT_PRESET_ID).unwrap();

        assert_eq!(warning.config.raw, MarkdownLintConfig::default().raw);
        assert_eq!(strict.config.raw, MarkdownLintConfig::default().raw);
        assert!(
            warning
                .rule_severity
                .values()
                .all(|it| *it == RuleSeverity::Warning)
        );
        assert!(
            strict
                .rule_severity
                .values()
                .all(|it| *it == RuleSeverity::Error)
        );
    }
}
