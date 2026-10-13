use super::MigrationStrategy;
use serde_json::{Value, json};

/* WHY: Migrates settings from 0.2.0 to 0.2.1.

Changes:
1. Renames `activity_rail_order` item `WorkspaceToggle` to `ExplorerToggle`.
2. Updates version to 0.2.1. */
pub struct Migration020To021;

impl MigrationStrategy for Migration020To021 {
    fn version(&self) -> &str {
        "0.2.0"
    }

    fn migrate(&self, mut json: Value) -> Value {
        let Some(obj) = json.as_object_mut() else {
            return json;
        };

        if let Some(layout) = obj.get_mut("layout").and_then(|v| v.as_object_mut())
            && let Some(rail) = layout
                .get_mut("activity_rail_order")
                .and_then(|v| v.as_array_mut())
        {
            for item in rail {
                if let Some(s) = item.as_str()
                    && s == "WorkspaceToggle"
                {
                    *item = json!("ExplorerToggle");
                }
            }
        }

        obj.insert("version".to_string(), json!("0.2.1"));
        json
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_from_0_2_0() {
        let strategy = Migration020To021;
        let old = json!({
            "version": "0.2.0",
            "layout": {
                "activity_rail_order": ["WorkspaceToggle", "Search"]
            }
        });
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["version"], "0.2.1");
        assert_eq!(
            migrated["layout"]["activity_rail_order"][0],
            "ExplorerToggle"
        );
        assert_eq!(migrated["layout"]["activity_rail_order"][1], "Search");
    }
}
