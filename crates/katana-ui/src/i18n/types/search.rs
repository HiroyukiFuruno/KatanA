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
    #[serde(default = "default_doc_search_no_results")]
    pub doc_search_no_results: String,
    pub tab_file_name: String,
    pub tab_markdown_content: String,
    pub md_query_hint: String,
    pub doc_query_hint: String,
    pub recent_searches: String,
    pub clear_history: String,
    pub ln_prefix: String,
    pub palette_query_hint: String,
    #[serde(default = "default_palette_action_query_hint")]
    pub palette_action_query_hint: String,
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
    #[serde(default = "default_command_toggle_split")]
    pub command_toggle_split: String,
    #[serde(default = "default_command_toggle_code_preview")]
    pub command_toggle_code_preview: String,
    /* WHY: Markdown authoring command labels shown in the command palette. */
    #[serde(default = "default_command_author_bold")]
    pub command_author_bold: String,
    #[serde(default = "default_command_author_italic")]
    pub command_author_italic: String,
    #[serde(default = "default_command_author_strikethrough")]
    pub command_author_strikethrough: String,
    #[serde(default = "default_command_author_inline_code")]
    pub command_author_inline_code: String,
    #[serde(default = "default_command_author_heading1")]
    pub command_author_heading1: String,
    #[serde(default = "default_command_author_heading2")]
    pub command_author_heading2: String,
    #[serde(default = "default_command_author_heading3")]
    pub command_author_heading3: String,
    #[serde(default = "default_command_author_bullet_list")]
    pub command_author_bullet_list: String,
    #[serde(default = "default_command_author_numbered_list")]
    pub command_author_numbered_list: String,
    #[serde(default = "default_command_author_blockquote")]
    pub command_author_blockquote: String,
    #[serde(default = "default_command_author_code_block")]
    pub command_author_code_block: String,
    #[serde(default = "default_command_author_horizontal_rule")]
    pub command_author_horizontal_rule: String,
    #[serde(default = "default_command_author_insert_link")]
    pub command_author_insert_link: String,
    #[serde(default = "default_command_author_insert_table")]
    pub command_author_insert_table: String,
    /* WHY: Image ingest command labels. */
    #[serde(default = "default_command_ingest_image_file")]
    pub command_ingest_image_file: String,
    #[serde(default = "default_command_ingest_clipboard_image")]
    pub command_ingest_clipboard_image: String,
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
fn default_command_toggle_split() -> String {
    "Toggle Split Mode".to_string()
}
fn default_command_toggle_code_preview() -> String {
    "Toggle Code Preview".to_string()
}
fn default_doc_search_no_results() -> String {
    "No results found.".to_string()
}
/* WHY: Authoring command defaults (English fallback for locales not yet updated). */
fn default_command_author_bold() -> String {
    "Bold".to_string()
}
fn default_command_author_italic() -> String {
    "Italic".to_string()
}
fn default_command_author_strikethrough() -> String {
    "Strikethrough".to_string()
}
fn default_command_author_inline_code() -> String {
    "Inline Code".to_string()
}
fn default_command_author_heading1() -> String {
    "Heading 1".to_string()
}
fn default_command_author_heading2() -> String {
    "Heading 2".to_string()
}
fn default_command_author_heading3() -> String {
    "Heading 3".to_string()
}
fn default_command_author_bullet_list() -> String {
    "Bullet List".to_string()
}
fn default_command_author_numbered_list() -> String {
    "Numbered List".to_string()
}
fn default_command_author_blockquote() -> String {
    "Blockquote".to_string()
}
fn default_command_author_code_block() -> String {
    "Code Block".to_string()
}
fn default_command_author_horizontal_rule() -> String {
    "Horizontal Rule".to_string()
}
fn default_command_author_insert_link() -> String {
    "Insert Link".to_string()
}
fn default_command_author_insert_table() -> String {
    "Insert Table".to_string()
}
fn default_command_ingest_image_file() -> String {
    "Attach Image File…".to_string()
}
fn default_command_ingest_clipboard_image() -> String {
    "Paste Image from Clipboard".to_string()
}
fn default_palette_action_query_hint() -> String {
    "Search Katana commands...".to_string()
}
