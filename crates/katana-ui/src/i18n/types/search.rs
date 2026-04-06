use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchMessages {
    pub modal_title: String,
    pub query_hint: String,
    pub include_pattern_hint: String,
    pub exclude_pattern_hint: String,
    pub no_results: String,
    #[serde(default = "default_palette_query_hint")]
    pub palette_query_hint: String,
    #[serde(default = "default_palette_no_results")]
    pub palette_no_results: String,
    #[serde(default = "default_doc_search_title")]
    pub doc_search_title: String,
    #[serde(default = "default_doc_search_prev")]
    pub doc_search_prev: String,
    #[serde(default = "default_doc_search_next")]
    pub doc_search_next: String,
    #[serde(default = "default_doc_search_close")]
    pub doc_search_close: String,
    #[serde(default = "default_doc_search_count")]
    pub doc_search_count: String,
    #[serde(default = "default_tab_file_name")]
    pub tab_file_name: String,
    #[serde(default = "default_tab_markdown_content")]
    pub tab_markdown_content: String,
    #[serde(default = "default_md_query_hint")]
    pub md_query_hint: String,
    #[serde(default = "default_recent_searches")]
    pub recent_searches: String,
    #[serde(default = "default_clear_history")]
    pub clear_history: String,
    #[serde(default = "default_ln_prefix")]
    pub ln_prefix: String,
    #[serde(default = "default_command_settings")]
    pub command_settings: String,
    #[serde(default = "default_command_explorer")]
    pub command_explorer: String,
    #[serde(default = "default_command_close_all")]
    pub command_close_all: String,
    #[serde(default = "default_command_refresh_explorer")]
    pub command_refresh_explorer: String,
    #[serde(default = "default_command_updates")]
    pub command_updates: String,
    #[serde(default = "default_command_about")]
    pub command_about: String,
    #[serde(default = "default_command_type_action")]
    pub command_type_action: String,
}

pub(super) fn default_command_settings() -> String {
    "Toggle Settings".to_string()
}
pub(super) fn default_command_explorer() -> String {
    "Toggle Workspace Panel".to_string()
}
pub(super) fn default_command_close_all() -> String {
    "Close All Documents".to_string()
}
pub(super) fn default_command_refresh_explorer() -> String {
    "Refresh Workspace".to_string()
}
pub(super) fn default_command_updates() -> String {
    "Check for Updates".to_string()
}
pub(super) fn default_command_about() -> String {
    "Toggle About".to_string()
}
pub(super) fn default_command_type_action() -> String {
    "Command".to_string()
}
pub(super) fn default_palette_query_hint() -> String {
    "Type a command or search...".to_string()
}
pub(super) fn default_tab_file_name() -> String {
    "File Name".to_string()
}
pub(super) fn default_tab_markdown_content() -> String {
    "Markdown Content".to_string()
}
pub(super) fn default_md_query_hint() -> String {
    "Search markdown files...".to_string()
}
pub(super) fn default_recent_searches() -> String {
    "Recent Searches".to_string()
}
pub(super) fn default_clear_history() -> String {
    "Clear".to_string()
}
pub(super) fn default_ln_prefix() -> String {
    "Ln ".to_string()
}
pub(super) fn default_palette_no_results() -> String {
    "No results found.".to_string()
}
pub(super) fn default_doc_search_title() -> String {
    "Search Document".to_string()
}
pub(super) fn default_doc_search_count() -> String {
    "{index}/{total}".to_string()
}
pub(super) fn default_doc_search_prev() -> String {
    "Previous".to_string()
}
pub(super) fn default_doc_search_next() -> String {
    "Next".to_string()
}
pub(super) fn default_doc_search_close() -> String {
    "Close".to_string()
}
