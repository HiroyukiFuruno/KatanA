use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMessages {
    pub modal_title: String,
    pub query_hint: String,
    pub include_pattern_hint: String,
    pub exclude_pattern_hint: String,
    pub no_results: String,
    pub palette_no_results: String,
    pub doc_search_title: String,
    pub doc_search_prev: String,
    pub doc_search_next: String,
    pub doc_search_close: String,
    pub doc_search_count: String,
    pub tab_file_name: String,
    pub tab_markdown_content: String,
    pub md_query_hint: String,
    pub recent_searches: String,
    pub clear_history: String,
    pub ln_prefix: String,
    pub palette_query_hint: String,
    pub command_settings: String,
    pub command_explorer: String,
    pub command_close_all: String,
    pub command_refresh_explorer: String,
    pub command_updates: String,
    pub command_about: String,
    pub command_type_action: String,
    #[serde(default = "default_command_global_search")]
    pub command_global_search: String,
    #[serde(default = "default_command_doc_search")]
    pub command_doc_search: String,
    #[serde(default = "default_command_refresh_document")]
    pub command_refresh_document: String,
}

fn default_command_global_search() -> String {
    "Global Search".to_string()
}
fn default_command_doc_search() -> String {
    "Document Search".to_string()
}
fn default_command_refresh_document() -> String {
    "Refresh Document".to_string()
}
