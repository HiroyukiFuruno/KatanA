use serde_json::Value;

pub trait MigrationStrategy: Send + Sync {
    fn version(&self) -> &str;
    fn migrate(&self, json: Value) -> Value;
}

pub struct MigrationRunner {
    pub strategies: Vec<Box<dyn MigrationStrategy>>,
}

pub struct V012Migration;
pub struct V013To014Migration;
pub struct V014To020Migration;
