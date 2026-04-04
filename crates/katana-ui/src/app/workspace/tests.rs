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
