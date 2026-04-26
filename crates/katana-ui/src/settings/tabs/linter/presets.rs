use katana_platform::settings::LinterPreset;
use katana_platform::settings::LinterSettings;
use katana_platform::settings::PresetReference;
use katana_platform::settings::PresetSource;
use std::path::Path;

use super::preset_configs::{
    DISABLED_PRESET_ID, KATANA_PRESET_ID, LinterPresetConfigOps, STRICT_PRESET_ID,
    WARNING_PRESET_ID,
};

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
        target_path: &Path,
    ) -> bool {
        match reference.source {
            PresetSource::BuiltIn => {
                Self::apply_built_in(settings, &reference.id, &reference.label, target_path)
            }
            PresetSource::User => Self::apply_user(settings, &reference.id),
            PresetSource::Custom | PresetSource::Unknown => false,
        }
    }

    pub(crate) fn apply_base(settings: &mut LinterSettings, target_path: &Path) -> bool {
        let Some(base) = settings.preset_state.base.clone() else {
            return Self::apply_built_in(settings, KATANA_PRESET_ID, "KatanA", target_path);
        };
        Self::apply_reference(settings, &base, target_path)
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

    fn apply_built_in(
        settings: &mut LinterSettings,
        id: &str,
        label: &str,
        target_path: &Path,
    ) -> bool {
        let Some(preset) = LinterPresetConfigOps::built_in(id) else {
            return false;
        };
        if !Self::save_markdownlint_config(&preset.config, target_path) {
            return false;
        }
        settings.rule_severity = preset.rule_severity;
        settings.preset_state.select_built_in(id, label);
        Self::sync_user_preset_state(settings);
        true
    }

    fn save_markdownlint_config(
        config: &katana_markdown_linter::MarkdownLintConfig,
        target_path: &Path,
    ) -> bool {
        if let Some(parent) = target_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            && std::fs::create_dir_all(parent).is_err()
        {
            return false;
        }
        config.save(target_path).is_ok()
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
}
