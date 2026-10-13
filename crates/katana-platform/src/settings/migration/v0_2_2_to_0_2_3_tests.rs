use super::MigrationStrategy;
use super::v0_2_2_to_0_2_3::Migration022To023;
use serde_json::json;

fn value_at<'a>(
    value: &'a serde_json::Value,
    path: &[&str],
) -> Result<&'a serde_json::Value, String> {
    path.iter().try_fold(value, |current, segment| {
        current
            .get(segment)
            .ok_or_else(|| format!("missing JSON path segment {segment:?} in {path:?}"))
    })
}

#[test]
fn migrates_preset_state_without_losing_existing_values() -> Result<(), String> {
    let old = json!({
        "version": "0.2.2",
        "theme": {
            "theme": "dark",
            "icon_pack": "lucide",
            "preset": "KatanaDark",
            "custom_color_overrides": { "name": "custom" },
            "custom_themes": [{ "name": "My Theme", "colors": { "name": "x" } }],
            "active_custom_theme": "My Theme"
        },
        "icon": {
            "active_preset": "My Icons",
            "active_overrides": { "ui/settings": { "vendor": "feather" } },
            "custom_presets": [{ "name": "My Icons", "overrides": {} }]
        },
        "linter": {
            "enabled": true,
            "use_workspace_local_config": true,
            "rule_severity": { "MD013": "ignore" }
        }
    });

    let migrated = Migration022To023.migrate(old);
    assert_eq!(value_at(&migrated, &["version"])?, "0.2.3");
    assert_eq!(
        value_at(&migrated, &["theme", "active_custom_theme"])?,
        "My Theme"
    );
    assert_eq!(
        value_at(&migrated, &["theme", "preset_state", "current", "source"])?,
        "custom"
    );
    assert_eq!(
        value_at(&migrated, &["theme", "preset_state", "base", "id"])?,
        "KatanaDark"
    );
    assert_eq!(
        value_at(&migrated, &["theme", "preset_state", "modified"])?,
        &json!(true)
    );
    assert_eq!(value_at(&migrated, &["icon", "active_preset"])?, "My Icons");
    assert_eq!(
        value_at(&migrated, &["icon", "preset_state", "current", "source"])?,
        "user"
    );
    assert_eq!(
        value_at(&migrated, &["icon", "preset_state", "modified"])?,
        &json!(true)
    );
    assert_eq!(
        value_at(&migrated, &["linter", "rule_severity", "MD013"])?,
        "ignore"
    );
    assert_eq!(
        value_at(&migrated, &["linter", "preset_state", "base", "id"])?,
        "katana"
    );
    assert_eq!(
        value_at(&migrated, &["linter", "preset_state", "modified"])?,
        &json!(true)
    );
    Ok(())
}

#[test]
fn migrates_unmodified_builtin_state_as_not_modified() -> Result<(), String> {
    let old = json!({
        "version": "0.2.2",
        "theme": { "preset": "KatanaLight", "icon_pack": "katana" },
        "icon": {},
        "linter": {}
    });

    let migrated = Migration022To023.migrate(old);
    assert_eq!(
        value_at(&migrated, &["theme", "preset_state", "current", "id"])?,
        "KatanaLight"
    );
    assert_eq!(
        value_at(&migrated, &["theme", "preset_state", "modified"])?,
        &json!(false)
    );
    assert_eq!(
        value_at(&migrated, &["icon", "preset_state", "current", "id"])?,
        "katana"
    );
    assert_eq!(
        value_at(&migrated, &["icon", "preset_state", "modified"])?,
        &json!(false)
    );
    assert_eq!(
        value_at(&migrated, &["linter", "preset_state", "current", "id"])?,
        "katana"
    );
    assert_eq!(
        value_at(&migrated, &["linter", "preset_state", "modified"])?,
        &json!(false)
    );
    Ok(())
}
