use super::MigrationStrategy;
use serde_json::{Value, json};

/* WHY: Migrates settings from 0.2.1 to 0.2.2.

Changes:
1. Updates version to 0.2.2 (shortcuts schema added). */
pub struct Migration021To022;

impl MigrationStrategy for Migration021To022 {
    fn version(&self) -> &str {
        "0.2.1"
    }

    fn migrate(&self, mut json: Value) -> Value {
        let Some(obj) = json.as_object_mut() else {
            return json;
        };

        obj.insert("version".to_string(), json!("0.2.2"));
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_from_0_2_1() {
        let strategy = Migration021To022;
        let old = json!({
            "version": "0.2.1",
            "layout": {}
        });
        let migrated = strategy.migrate(old);
        assert_eq!(migrated["version"], "0.2.2");
    }
}
