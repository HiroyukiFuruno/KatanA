use super::MigrationStrategy;
use serde_json::{Value, json};

/* WHY: Migrates settings from 0.2.1 to 0.2.2.

Changes:
1. Replaces `workspace.paths` with `workspace.persisted` and `workspace.histories`.
2. Inserts `AddWorkspace` at the beginning of `activity_rail_order`.
3. Updates version to 0.2.2. */
pub struct Migration021To022;

impl MigrationStrategy for Migration021To022 {
    fn version(&self) -> &str {
        "0.2.1"
    }

    fn migrate(&self, mut json: Value) -> Value {
        let Some(obj) = json.as_object_mut() else {
            return json;
        };

        if let Some(workspace) = obj.get_mut("workspace").and_then(|v| v.as_object_mut())
            && let Some(paths) = workspace.remove("paths")
            && let Some(paths_array) = paths.as_array()
        {
            workspace.insert("persisted".to_string(), json!(paths_array));
            workspace.insert("histories".to_string(), json!(paths_array));
        }

        /* WHY: Check and update activity_rail_order for existing users to match the new defaults */
        if let Some(layout) = obj.get_mut("layout").and_then(|v| v.as_object_mut())
            && let Some(rail) = layout
                .get_mut("activity_rail_order")
                .and_then(|v| v.as_array_mut())
        {
            let has_add = rail.iter().any(|v| v.as_str() == Some("AddWorkspace"));
            let has_workspace = rail.iter().any(|v| v.as_str() == Some("WorkspaceToggle"));

            if !has_add {
                rail.insert(0, json!("AddWorkspace"));
            }
            if !has_workspace {
                let insert_idx = rail
                    .iter()
                    .position(|v| v.as_str() == Some("ExplorerToggle"))
                    .unwrap_or(0);
                rail.insert(insert_idx, json!("WorkspaceToggle"));
            }
        }

        obj.insert("version".to_string(), json!("0.2.2"));
        json
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_from_0_2_1() {
        let strategy = Migration021To022;
        let old = json!({
            "version": "0.2.1",
            "workspace": {
                "paths": ["/tmp/a", "/tmp/b"]
            }
        });
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["version"], "0.2.2");
        assert_eq!(migrated["workspace"]["persisted"][0], "/tmp/a");
        assert_eq!(migrated["workspace"]["histories"][1], "/tmp/b");
        assert!(
            migrated["workspace"]
                .as_object()
                .unwrap()
                .get("paths")
                .is_none()
        );
    }

    #[test]
    fn test_migrate_adds_add_workspace_to_rail() {
        let strategy = Migration021To022;
        let old = json!({
            "version": "0.2.1",
            "layout": {
                "activity_rail_order": ["ExplorerToggle", "Search", "History", "Settings"]
            }
        });
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["layout"]["activity_rail_order"][0], "AddWorkspace");
        assert_eq!(
            migrated["layout"]["activity_rail_order"][1],
            "WorkspaceToggle"
        );
        assert_eq!(
            migrated["layout"]["activity_rail_order"][2],
            "ExplorerToggle"
        );
    }
}
