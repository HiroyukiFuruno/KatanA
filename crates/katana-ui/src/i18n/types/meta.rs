use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaInfoMessages {
    pub title: String,
    pub general_section: String,
    pub dates_section: String,
    pub status_section: String,
    pub label_name: String,
    pub label_kind: String,
    pub label_version: String,
    pub label_created: String,
    pub label_updated: String,
    pub label_dirty: String,
    pub label_loaded: String,
    pub label_pinned: String,
    pub label_path: String,
    pub kind_markdown: String,
    pub yes: String,
    pub no: String,
    #[serde(default = "default_meta_label_size")]
    pub label_size: String,
    #[serde(default = "default_meta_label_owner")]
    pub label_owner: String,
    #[serde(default = "default_meta_label_permissions")]
    pub label_permissions: String,
    #[serde(default = "default_meta_kind_virtual")]
    pub kind_virtual: String,
    #[serde(default = "default_meta_virtual_embedded")]
    pub virtual_embedded: String,
    #[serde(default = "default_meta_unknown")]
    pub unknown: String,
}

fn default_meta_kind_virtual() -> String {
    "Virtual Document".to_string()
}
fn default_meta_virtual_embedded() -> String {
    "Embedded".to_string()
}
fn default_meta_unknown() -> String {
    "Unknown".to_string()
}

fn default_meta_label_size() -> String {
    "File Size".to_string()
}
fn default_meta_label_owner() -> String {
    "Owner".to_string()
}
fn default_meta_label_permissions() -> String {
    "Permissions".to_string()
}
