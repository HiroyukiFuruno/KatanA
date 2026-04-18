/* WHY: Verification of default values and fallbacks in settings structures. */

use super::*;

#[test]
fn test_layout_settings_default_deserialization() {
    let json = "{}";
    let layout: LayoutSettings = serde_json::from_str(json).unwrap();
    assert!(layout.toc_visible);
}

#[test]
fn test_workspace_settings_default_deserialization() {
    let json = "{}";
    let ws: WorkspaceSettings = serde_json::from_str(json).unwrap();
    assert_eq!(ws.max_depth, DEFAULT_MAX_DEPTH);
    assert!(!ws.visible_extensions.is_empty());
    assert!(!ws.extensionless_excludes.is_empty());
    assert!(!ws.ignored_directories.is_empty());
}

#[test]
fn test_app_settings_default_values() {
    let s = AppSettings::default();
    assert_eq!(s.theme.theme, SettingsDefaultOps::default_theme());
    assert_eq!(s.theme.preset, SettingsDefaultOps::select_initial_preset());
    assert!(s.theme.custom_color_overrides.is_none());
    assert!((s.font.size - 14.0).abs() < f32::EPSILON);
    assert_eq!(s.font.family, "monospace");
    assert_eq!(s.language, "en");
    assert!(s.workspace.last_workspace.is_none());
    /* WHY: Behavior defaults */
    assert!(s.behavior.confirm_close_dirty_tab);
    assert!(s.behavior.scroll_sync_enabled);
    assert!(s.behavior.auto_save);
    assert_eq!(s.behavior.auto_save_interval_secs, 5.0);
    assert!(s.behavior.auto_refresh);
    assert_eq!(s.behavior.auto_refresh_interval_secs, 2.0);
}

#[test]
fn test_behavior_settings_defaults() {
    let b = BehaviorSettings::default();
    assert!(b.confirm_close_dirty_tab);
    assert!(b.scroll_sync_enabled);
    assert!(b.auto_save);
    assert_eq!(b.auto_save_interval_secs, 5.0);
    assert!(b.auto_refresh);
    assert_eq!(b.auto_refresh_interval_secs, 2.0);
}

#[test]
fn test_split_direction_defaults_to_horizontal() {
    let settings = AppSettings::default();
    assert_eq!(settings.layout.split_direction, SplitDirection::Horizontal);
}

#[test]
fn test_pane_order_defaults_to_editor_first() {
    let settings = AppSettings::default();
    assert_eq!(settings.layout.pane_order, PaneOrder::EditorFirst);
}
