use std::hash::{Hash, Hasher};

pub(crate) struct LinterDiagnosticContextOps;

impl LinterDiagnosticContextOps {
    pub(crate) fn hash(
        state: &crate::app_state::AppState,
        options: &katana_markdown_linter::LintOptions,
    ) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let linter_settings = &state.config.settings.settings().linter;
        linter_settings.enabled.hash(&mut hasher);
        linter_settings.use_workspace_local_config.hash(&mut hasher);
        Self::hash_lint_options(options, &mut hasher);
        let mut severities = linter_settings.rule_severity.iter().collect::<Vec<_>>();
        severities.sort_by_key(|(rule_id, _)| *rule_id);
        for (rule_id, severity) in severities {
            rule_id.hash(&mut hasher);
            Self::hash_rule_severity(severity, &mut hasher);
        }
        hasher.finish()
    }

    fn hash_lint_options(options: &katana_markdown_linter::LintOptions, hasher: &mut impl Hasher) {
        Self::hash_severity(options.default_severity, hasher);
        let mut rules = options.rules.iter().collect::<Vec<_>>();
        rules.sort_by_key(|(rule_id, _)| *rule_id);
        for (rule_id, rule_config) in rules {
            rule_id.hash(hasher);
            rule_config.enabled.hash(hasher);
            let mut properties = rule_config.properties.iter().collect::<Vec<_>>();
            properties.sort_by_key(|(key, _)| *key);
            for (key, value) in properties {
                key.hash(hasher);
                value.hash(hasher);
            }
        }
    }

    fn hash_severity(severity: katana_markdown_linter::Severity, hasher: &mut impl Hasher) {
        let value = match severity {
            katana_markdown_linter::Severity::Error => 0_u8,
            katana_markdown_linter::Severity::Warning => 1_u8,
            katana_markdown_linter::Severity::Info => 2_u8,
        };
        value.hash(hasher);
    }

    fn hash_rule_severity(
        severity: &katana_platform::settings::types::RuleSeverity,
        hasher: &mut impl Hasher,
    ) {
        let value = match severity {
            katana_platform::settings::types::RuleSeverity::Ignore => 0_u8,
            katana_platform::settings::types::RuleSeverity::Warning => 1_u8,
            katana_platform::settings::types::RuleSeverity::Error => 2_u8,
        };
        value.hash(hasher);
    }
}

#[cfg(test)]
mod tests {
    use super::LinterDiagnosticContextOps;
    use katana_markdown_linter::{LintOptions, RuleConfig, Severity};
    use katana_platform::settings::types::RuleSeverity;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_state() -> crate::app_state::AppState {
        crate::app_state::AppState::new(
            Default::default(),
            Default::default(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        )
    }

    #[test]
    fn hash_changes_with_severity_and_rule_overrides() {
        let mut state = make_state();
        state
            .config
            .settings
            .settings_mut()
            .linter
            .rule_severity
            .insert("MD001".to_string(), RuleSeverity::Ignore);
        state
            .config
            .settings
            .settings_mut()
            .linter
            .rule_severity
            .insert("MD002".to_string(), RuleSeverity::Warning);
        state
            .config
            .settings
            .settings_mut()
            .linter
            .rule_severity
            .insert("MD003".to_string(), RuleSeverity::Error);
        let mut options = LintOptions {
            default_severity: Severity::Error,
            rules: HashMap::from([(
                "MD001".to_string(),
                RuleConfig {
                    enabled: true,
                    properties: HashMap::from([("style".to_string(), "atx".to_string())]),
                },
            )]),
        };

        let error_hash = LinterDiagnosticContextOps::hash(&state, &options);
        options.default_severity = Severity::Info;
        let info_hash = LinterDiagnosticContextOps::hash(&state, &options);

        assert_ne!(error_hash, info_hash);
    }
}
