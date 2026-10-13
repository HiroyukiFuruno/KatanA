use super::*;
use serde_json::json;

struct DummyStrategy {
    version: String,
}

impl MigrationStrategy for DummyStrategy {
    fn version(&self) -> &str {
        &self.version
    }

    fn migrate(&self, mut json: Value) -> Value {
        json.as_object_mut()
            .unwrap()
            .insert("version".to_string(), json!("next"));
        json
    }
}

#[test]
fn test_migration_runner_default() {
    let runner = MigrationRunner::default();
    assert!(runner.strategies.is_empty());
}

#[test]
fn test_migration_runner_loop_and_unmapped() {
    let mut runner = MigrationRunner::new();
    runner.add_strategy(Box::new(DummyStrategy {
        version: "0.1.2".to_string(),
    }));

    let initial_json = json!({"version": "0.1.2"});
    let migrated_json = runner.migrate(initial_json);
    assert_eq!(
        migrated_json.get("version").unwrap().as_str().unwrap(),
        "next"
    );
}
