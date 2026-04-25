/* WHY: Verification of settings persistence and version compatibility. */

use super::*;

#[test]
fn test_behavior_settings_serde_roundtrip() {
    let b = BehaviorSettings {
        confirm_close_dirty_tab: false,
        scroll_sync_enabled: false,
        auto_save: true,
        auto_save_interval_secs: 10.0,
        auto_refresh: false,
        auto_refresh_interval_secs: 10.0,
        confirm_file_move: false,
        slideshow_hover_highlight: true,
        slideshow_show_diagram_controls: true,
    };
    let json = serde_json::to_string(&b).unwrap();
    let loaded: BehaviorSettings = serde_json::from_str(&json).unwrap();
    assert!(!loaded.confirm_close_dirty_tab);
    assert!(!loaded.scroll_sync_enabled);
    assert!(loaded.auto_save);
    assert_eq!(loaded.auto_save_interval_secs, 10.0);
    assert!(!loaded.auto_refresh);
    assert_eq!(loaded.auto_refresh_interval_secs, 10.0);
    assert!(!loaded.confirm_file_move);
}

#[test]
fn test_behavior_settings_serde_missing_fields_use_defaults() {
    let json = "{}";
    let loaded: BehaviorSettings = serde_json::from_str(json).unwrap();
    assert!(loaded.confirm_close_dirty_tab);
    assert!(loaded.scroll_sync_enabled);
    assert!(loaded.auto_save);
    assert_eq!(loaded.auto_save_interval_secs, 5.0);
    assert!(loaded.auto_refresh);
    assert_eq!(loaded.auto_refresh_interval_secs, 2.0);
    assert!(loaded.confirm_file_move);
}

#[test]
fn test_app_settings_serde_roundtrip() {
    let mut s = AppSettings {
        theme: ThemeSettings {
            theme: "light".to_string(),
            ..Default::default()
        },
        font: FontSettings {
            size: 16.0,
            ..Default::default()
        },
        ..Default::default()
    };
    s.extra.push(ExtraSetting {
        key: "key".to_string(),
        value: "value".to_string(),
    });

    let json = serde_json::to_string(&s).unwrap();
    let loaded: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.theme.theme, "light");
    assert!((loaded.font.size - 16.0).abs() < f32::EPSILON);
    let ext = loaded.extra.iter().find(|e| e.key == "key").unwrap();
    assert_eq!(ext.value, "value");
}

#[test]
fn test_app_settings_serde_missing_fields_use_defaults() {
    let json = r#"{"theme": {"theme": "custom"}}"#;
    let loaded: AppSettings = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.theme.theme, "custom");
    assert!((loaded.font.size - 14.0).abs() < f32::EPSILON);
    assert_eq!(loaded.language, "en");
}

#[test]
fn test_behavior_settings_fractional_auto_save_interval() {
    let mut b = BehaviorSettings {
        auto_save_interval_secs: 5.1,
        ..Default::default()
    };

    let json = serde_json::to_string(&b).unwrap();
    assert!(
        json.contains("5.1"),
        "Should serialize as float with exactly 1 decimal representation for 0.1s"
    );

    let parsed: BehaviorSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed.auto_save_interval_secs, 5.1,
        "Must roundtrip 0.1 float boundaries precisely to support egui interval sliding"
    );

    /* WHY: Edge boundary testing */
    b.auto_save_interval_secs = 0.0;
    let parsed: BehaviorSettings =
        serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
    assert_eq!(
        parsed.auto_save_interval_secs, 0.0,
        "Zero boundary strict matching"
    );

    b.auto_save_interval_secs = 300.0;
    let parsed: BehaviorSettings =
        serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
    assert_eq!(
        parsed.auto_save_interval_secs, 300.0,
        "Max boundary strict matching"
    );
}

#[test]
fn test_layout_settings_serde_backward_compat() {
    let json = r#"{"theme": {"theme": "dark"}}"#;
    let loaded: AppSettings = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.layout.split_direction, SplitDirection::Horizontal);
    assert_eq!(loaded.layout.pane_order, PaneOrder::EditorFirst);
}

#[test]
fn test_layout_settings_roundtrip() {
    let mut settings = AppSettings::default();
    settings.layout.split_direction = SplitDirection::Vertical;
    settings.layout.pane_order = PaneOrder::PreviewFirst;

    let json = serde_json::to_string(&settings).unwrap();
    let loaded: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.layout.split_direction, SplitDirection::Vertical);
    assert_eq!(loaded.layout.pane_order, PaneOrder::PreviewFirst);
}

#[test]
fn test_skipped_version_backward_compat() {
    let json = r#"{"updates": {"interval": "Daily"}}"#;
    let loaded: AppSettings = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.updates.skipped_version, None);
    assert_eq!(loaded.updates.previous_app_version, None);
}

#[test]
fn test_new_format_theme_colors_roundtrip() {
    let preset = ThemePreset::KatanaDark;
    let original = preset.colors();
    let json = serde_json::to_string(&original).unwrap();
    let roundtripped: ThemeColors = serde_json::from_str(&json).unwrap();
    assert_eq!(original, roundtripped);
}
