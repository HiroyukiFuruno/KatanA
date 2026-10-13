pub mod types;
pub mod v0_1_2;
pub mod v0_1_3_to_0_1_4;
pub mod v0_1_4_to_0_2_0;

pub mod v0_2_0_to_0_2_1;
pub mod v0_2_1_to_0_2_2;
pub mod v0_2_2_to_0_2_3;

#[cfg(test)]
mod v0_2_2_to_0_2_3_tests;

use serde_json::Value;
pub use types::*;

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn MigrationStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn migrate(&self, mut json: Value) -> Value {
        loop {
            let current_version = json
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.2")
                .to_string();

            let mut mapped = false;
            for strategy in &self.strategies {
                if strategy.version() == current_version {
                    tracing::info!("Migrating settings from version: {}", current_version);
                    json = strategy.migrate(json);
                    mapped = true;
                    break;
                }
            }
            if !mapped {
                break;
            }
        }
        json
    }
}

#[cfg(test)]
mod tests;
