use super::presets::LinterPresetOps;
use katana_platform::settings::{LinterSettings, PresetReference, RuleSeverity};

#[test]
fn strict_builtin_preset_sets_all_rules_to_error() {
    let mut settings = LinterSettings::default();
    let reference = PresetReference::built_in("strict", "Strict");
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let target_path = dir.path().join(".markdownlint.json");

    assert!(LinterPresetOps::apply_reference(
        &mut settings,
        &reference,
        &target_path
    ));

    assert!(!settings.rule_severity.is_empty());
    assert!(
        settings
            .rule_severity
            .values()
            .all(|severity| *severity == RuleSeverity::Error)
    );
    assert_eq!(
        settings
            .preset_state
            .current
            .expect("current preset must exist")
            .id,
        "strict"
    );
    let saved = katana_markdown_linter::MarkdownLintConfig::load(&target_path)
        .expect("saved config must load");
    assert_eq!(
        saved.raw,
        katana_markdown_linter::MarkdownLintConfig::default().raw
    );
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

    let dir = tempfile::tempdir().expect("tempdir must be available");
    assert!(LinterPresetOps::apply_reference(
        &mut settings,
        &reference,
        &dir.path().join(".markdownlint.json")
    ));
    assert_eq!(
        settings.rule_severity.get("MD013"),
        Some(&RuleSeverity::Ignore)
    );
    assert_eq!(
        settings
            .preset_state
            .current
            .expect("current preset must exist")
            .id,
        "Team"
    );
    assert_eq!(settings.preset_state.user_presets.len(), 1);
}

#[test]
fn katana_builtin_preset_writes_bundled_rules_and_severity() {
    let mut settings = LinterSettings::default();
    let reference = PresetReference::built_in("katana", "KatanA");
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let target_path = dir.path().join(".markdownlint.json");

    assert!(LinterPresetOps::apply_reference(
        &mut settings,
        &reference,
        &target_path
    ));

    let saved = katana_markdown_linter::MarkdownLintConfig::load(&target_path)
        .expect("saved config must load");
    assert_eq!(saved.raw["MD013"], false);
    assert_eq!(
        settings.rule_severity.get("MD013"),
        Some(&RuleSeverity::Ignore)
    );
    assert_eq!(
        settings.rule_severity.get("MD048"),
        Some(&RuleSeverity::Error)
    );
    assert_eq!(
        settings.rule_severity.get("MD001"),
        Some(&RuleSeverity::Warning)
    );
}
