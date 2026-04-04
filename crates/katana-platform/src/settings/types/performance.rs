use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_cache_retention")]
    pub cache_retention_days: u32,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub optimize_for_speed: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::default_diagram_concurrency")]
    pub diagram_concurrency: usize,
}
