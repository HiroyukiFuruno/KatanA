use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchSettings {
    #[serde(default)]
    pub recent_md_queries: Vec<String>,
}
