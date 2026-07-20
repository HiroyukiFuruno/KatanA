use super::*;
use crate::state::document::TabGroup;

#[test]
fn test_v2() {
    let state = WorkspaceTabSessionV2 {
        version: 2,
        tabs: vec![],
        active_path: None,
        expanded_directories: std::collections::HashSet::new(),
        groups: vec![TabGroup {
            id: "id1".to_string(),
            name: "group1".to_string(),
            color_hex: "#123456".to_string(),
            collapsed: false,
            members: vec!["mem1".to_string()],
        }],
    };
    let json = serde_json::to_string(&state).unwrap();
    let parsed: WorkspaceTabSessionV2 = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.groups.len(), 1);
}

#[test]
fn workspace_tab_policy_adds_different_workspace_when_enabled() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_open(
        vec!["/work/a".to_string()],
        Some("/work/a".to_string()),
        "/work/b",
        true,
    );

    assert_eq!(result.open_tabs, vec!["/work/a", "/work/b"]);
    assert_eq!(result.active_workspace, Some("/work/b".to_string()));
}

#[test]
fn workspace_tab_policy_replaces_active_tab_when_disabled() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_open(
        vec!["/work/a".to_string()],
        Some("/work/a".to_string()),
        "/work/b",
        false,
    );

    assert_eq!(result.open_tabs, vec!["/work/b"]);
    assert_eq!(result.active_workspace, Some("/work/b".to_string()));
}

#[test]
fn workspace_tab_policy_does_not_grow_existing_multiple_tabs_when_disabled() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_open(
        vec![
            "/work/a".to_string(),
            "/work/b".to_string(),
            "/work/c".to_string(),
        ],
        Some("/work/b".to_string()),
        "/work/d",
        false,
    );

    assert_eq!(result.open_tabs, vec!["/work/a", "/work/d", "/work/c"]);
    assert_eq!(result.active_workspace, Some("/work/d".to_string()));
}

#[test]
fn workspace_tab_policy_closes_inactive_without_changing_active() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_close(
        vec!["/work/a".to_string(), "/work/b".to_string()],
        Some("/work/a".to_string()),
        "/work/b",
    );

    assert_eq!(result.open_tabs, vec!["/work/a"]);
    assert_eq!(result.active_workspace, Some("/work/a".to_string()));
}

#[test]
fn workspace_tab_policy_closes_active_and_selects_neighbor() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_close(
        vec![
            "/work/a".to_string(),
            "/work/b".to_string(),
            "/work/c".to_string(),
        ],
        Some("/work/b".to_string()),
        "/work/b",
    );

    assert_eq!(result.open_tabs, vec!["/work/a", "/work/c"]);
    assert_eq!(result.active_workspace, Some("/work/c".to_string()));
}

#[test]
fn workspace_tab_policy_closes_last_active_workspace() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_close(
        vec!["/work/a".to_string()],
        Some("/work/a".to_string()),
        "/work/a",
    );

    assert!(result.open_tabs.is_empty());
    assert!(result.active_workspace.is_none());
}

#[test]
fn workspace_tab_policy_reorders_tabs_without_changing_active_workspace() {
    let result = tabs::WorkspaceTabsOps::resolve_open_tabs_after_reorder(
        vec![
            "/work/a".to_string(),
            "/work/b".to_string(),
            "/work/c".to_string(),
        ],
        Some("/work/b".to_string()),
        0,
        3,
    );

    assert_eq!(result.open_tabs, vec!["/work/b", "/work/c", "/work/a"]);
    assert_eq!(result.active_workspace, Some("/work/b".to_string()));
}

#[test]
fn effective_visible_extensions_include_html_even_when_settings_are_empty() {
    let mut settings = katana_platform::settings::WorkspaceSettings::default();
    settings.visible_extensions.clear();

    let extensions = WorkspaceExtensionPolicy::effective(&settings);

    assert!(extensions.iter().any(|ext| ext == "html"));
    assert!(extensions.iter().any(|ext| ext == "htm"));
}

#[test]
fn file_creation_visible_extensions_include_standard_html_documents() {
    let settings = katana_platform::settings::WorkspaceSettings {
        visible_extensions: vec!["md".to_string()],
        ..Default::default()
    };

    let extensions = WorkspaceExtensionPolicy::file_creation(&settings);

    assert!(extensions.iter().any(|ext| ext == "html"));
    assert!(extensions.iter().any(|ext| ext == "htm"));
    assert!(
        !extensions.iter().any(|ext| ext == "png"),
        "new-file choices should not inherit every tree-visible binary extension"
    );
}
