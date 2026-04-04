use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchSettings {
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub favorite_queries: Vec<String>,
    #[serde(default)]
    pub recent_md_queries: Vec<String>,
}
