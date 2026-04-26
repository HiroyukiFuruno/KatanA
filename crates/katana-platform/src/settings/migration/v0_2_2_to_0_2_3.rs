use super::MigrationStrategy;
use serde_json::{Map, Value, json};

pub struct Migration022To023;

impl MigrationStrategy for Migration022To023 {
    fn version(&self) -> &str {
        "0.2.2"
    }

    fn migrate(&self, mut json: Value) -> Value {
        let Some(settings) = json.as_object_mut() else {
            return json;
        };

        migrate_theme(settings);
        migrate_icon(settings);
        migrate_linter(settings);
        settings.insert("version".to_string(), json!("0.2.3"));
        json
    }
}

fn migrate_theme(settings: &mut Map<String, Value>) {
    let Some(theme) = object_at(settings, "theme") else {
        return;
    };
    if theme.contains_key("preset_state") {
        return;
    }

    let preset = string_at(theme, "preset").unwrap_or_else(|| "KatanaDark".to_string());
    let custom_theme = string_at(theme, "active_custom_theme");
    let has_color_overrides = !theme
        .get("custom_color_overrides")
        .unwrap_or(&Value::Null)
        .is_null();

    let current = custom_theme
        .as_ref()
        .map(|name| reference("custom", name, name))
        .unwrap_or_else(|| reference("built_in", &preset, &preset));
    let user_presets = user_preset_names(theme, "custom_themes");

    theme.insert(
        "preset_state".to_string(),
        preset_state(
            current,
            reference("built_in", &preset, &preset),
            custom_theme.is_some() || has_color_overrides,
            user_presets,
        ),
    );
}

fn migrate_icon(settings: &mut Map<String, Value>) {
    let icon_pack = settings
        .get("theme")
        .and_then(Value::as_object)
        .and_then(|theme| string_at(theme, "icon_pack"))
        .unwrap_or_else(|| "katana".to_string());
    let Some(icon) = object_at(settings, "icon") else {
        return;
    };
    if icon.contains_key("preset_state") {
        return;
    }

    let active_preset = string_at(icon, "active_preset");
    let has_overrides = icon
        .get("active_overrides")
        .and_then(Value::as_object)
        .is_some_and(|overrides| !overrides.is_empty());
    let current = active_preset
        .as_ref()
        .map(|name| reference("user", name, name))
        .unwrap_or_else(|| reference("built_in", &icon_pack, &icon_pack));
    let base = active_preset
        .as_ref()
        .map(|name| reference("user", name, name))
        .unwrap_or_else(|| reference("built_in", &icon_pack, &icon_pack));
    let user_presets = user_preset_names(icon, "custom_presets");

    icon.insert(
        "preset_state".to_string(),
        preset_state(current, base, has_overrides, user_presets),
    );
}

fn migrate_linter(settings: &mut Map<String, Value>) {
    let Some(linter) = object_at(settings, "linter") else {
        return;
    };
    if linter.contains_key("preset_state") {
        return;
    }

    let enabled = linter
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let workspace_config = linter
        .get("use_workspace_local_config")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_rule_overrides = linter
        .get("rule_severity")
        .and_then(Value::as_object)
        .is_some_and(|rules| !rules.is_empty());

    linter.insert(
        "preset_state".to_string(),
        preset_state(
            reference("built_in", "katana", "KatanA"),
            reference("built_in", "katana", "KatanA"),
            !enabled || workspace_config || has_rule_overrides,
            Vec::new(),
        ),
    );
}

fn object_at<'a>(
    settings: &'a mut Map<String, Value>,
    key: &str,
) -> Option<&'a mut Map<String, Value>> {
    settings.get_mut(key)?.as_object_mut()
}

fn string_at(values: &Map<String, Value>, key: &str) -> Option<String> {
    values.get(key)?.as_str().map(ToOwned::to_owned)
}

fn user_preset_names(values: &Map<String, Value>, key: &str) -> Vec<Value> {
    values
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.get("name").and_then(Value::as_str))
        .map(|name| reference("user", name, name))
        .collect()
}

fn preset_state(current: Value, base: Value, modified: bool, user_presets: Vec<Value>) -> Value {
    json!({
        "current": current,
        "base": base,
        "modified": modified,
        "user_presets": user_presets,
    })
}

fn reference(source: &str, id: &str, label: &str) -> Value {
    json!({
        "source": source,
        "id": id,
        "label": label,
    })
}
