use katana_platform::settings::LinterPreset;
use katana_platform::settings::LinterSettings;
use katana_platform::settings::PresetReference;
use katana_platform::settings::PresetSource;
use katana_platform::settings::RuleSeverity;
use std::collections::HashMap;

const KATANA_PRESET_ID: &str = "katana";
const DISABLED_PRESET_ID: &str = "disabled";
const STRICT_PRESET_ID: &str = "strict";
const WARNING_PRESET_ID: &str = "warning";

pub(crate) struct LinterPresetOps;

impl LinterPresetOps {
    pub(crate) fn built_in_presets(
        labels: &crate::i18n::LinterTranslations,
    ) -> Vec<PresetReference> {
        vec![
            PresetReference::built_in(KATANA_PRESET_ID, &labels.preset_katana),
            PresetReference::built_in(DISABLED_PRESET_ID, &labels.preset_disabled),
            PresetReference::built_in(STRICT_PRESET_ID, &labels.preset_strict),
            PresetReference::built_in(WARNING_PRESET_ID, &labels.preset_warning),
        ]
    }

    pub(crate) fn user_preset_references(settings: &LinterSettings) -> Vec<PresetReference> {
        settings
            .custom_presets
            .iter()
            .map(|preset| PresetReference::user(&preset.name))
            .collect()
    }

    pub(crate) fn apply_reference(
        settings: &mut LinterSettings,
        reference: &PresetReference,
    ) -> bool {
        match reference.source {
            PresetSource::BuiltIn => {
                Self::apply_built_in(settings, &reference.id, &reference.label)
            }
            PresetSource::User => Self::apply_user(settings, &reference.id),
            PresetSource::Custom | PresetSource::Unknown => false,
        }
    }

    pub(crate) fn apply_base(settings: &mut LinterSettings) -> bool {
        let Some(base) = settings.preset_state.base.clone() else {
            return Self::apply_built_in(settings, KATANA_PRESET_ID, "KatanA");
        };
        Self::apply_reference(settings, &base)
    }

    pub(crate) fn save_current_as_user_preset(settings: &mut LinterSettings, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return;
        }
        let preset = LinterPreset {
            name: trimmed.to_string(),
            rule_severity: settings.rule_severity.clone(),
        };
        if let Some(existing) = settings
            .custom_presets
            .iter_mut()
            .find(|it| it.name == preset.name)
        {
            *existing = preset;
        } else {
            settings.custom_presets.push(preset);
        }
        settings.preset_state.select_user(trimmed);
        Self::sync_user_preset_state(settings);
    }

    pub(crate) fn mark_modified(settings: &mut LinterSettings) {
        settings.preset_state.mark_modified();
        Self::sync_user_preset_state(settings);
    }

    pub(crate) fn sync_user_preset_state(settings: &mut LinterSettings) {
        settings
            .preset_state
            .sync_user_preset_names(settings.custom_presets.iter().map(|preset| &preset.name));
    }

    fn apply_built_in(settings: &mut LinterSettings, id: &str, label: &str) -> bool {
        match id {
            KATANA_PRESET_ID => settings.rule_severity.clear(),
            DISABLED_PRESET_ID => settings.rule_severity = Self::all_rules(RuleSeverity::Ignore),
            STRICT_PRESET_ID => settings.rule_severity = Self::all_rules(RuleSeverity::Error),
            WARNING_PRESET_ID => settings.rule_severity = Self::all_rules(RuleSeverity::Warning),
            _ => return false,
        }
        settings.preset_state.select_built_in(id, label);
        Self::sync_user_preset_state(settings);
        true
    }

    fn apply_user(settings: &mut LinterSettings, name: &str) -> bool {
        let Some(preset) = settings
            .custom_presets
            .iter()
            .find(|preset| preset.name == name)
            .cloned()
        else {
            return false;
        };
        settings.rule_severity = preset.rule_severity;
        settings.preset_state.select_user(name);
        Self::sync_user_preset_state(settings);
        true
    }

    fn all_rules(severity: RuleSeverity) -> HashMap<String, RuleSeverity> {
        katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
            .into_iter()
            .map(|rule| (rule.id().to_string(), severity))
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn strict_builtin_preset_sets_all_rules_to_error() {
        let mut settings = LinterSettings::default();
        let reference = PresetReference::built_in(STRICT_PRESET_ID, "Strict");

        assert!(LinterPresetOps::apply_reference(&mut settings, &reference));

        assert!(!settings.rule_severity.is_empty());
        assert!(
            settings
                .rule_severity
                .values()
                .all(|severity| *severity == RuleSeverity::Error)
        );
        assert_eq!(settings.preset_state.current.unwrap().id, STRICT_PRESET_ID);
    }

    #[test]
    fn user_preset_round_trip_keeps_rule_severity() {
        let mut settings = LinterSettings::default();
        settings
            .rule_severity
            .insert("MD013".to_string(), RuleSeverity::Ignore);

        LinterPresetOps::save_current_as_user_preset(&mut settings, "Team");
        settings.rule_severity.clear();
        let reference = PresetReference::user("Team");

        assert!(LinterPresetOps::apply_reference(&mut settings, &reference));
        assert_eq!(
            settings.rule_severity.get("MD013"),
            Some(&RuleSeverity::Ignore)
        );
        assert_eq!(settings.preset_state.current.unwrap().id, "Team");
        assert_eq!(settings.preset_state.user_presets.len(), 1);
    }
}
