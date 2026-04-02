use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SearchTab {
    FileName,
    MarkdownContent,
}

pub struct SearchState {
    pub filter_enabled: bool,
    pub filter_query: String,
    pub filter_cache: Option<(String, HashSet<PathBuf>)>,

    // UI state
    pub active_tab: SearchTab,

    // File name search state
    pub query: String,
    pub include_pattern: String,
    pub exclude_pattern: String,
    pub last_params: Option<(String, String, String)>,
    pub results: Vec<PathBuf>,

    // Markdown content search state
    pub md_query: String,
    pub md_last_query: Option<String>,
    pub md_results: Vec<katana_core::search::SearchResult>,
    pub md_history: katana_core::search::SearchHistory,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            filter_enabled: false,
            filter_query: String::new(),
            filter_cache: None,

            active_tab: SearchTab::FileName,

            query: String::new(),
            include_pattern: String::new(),
            exclude_pattern: String::new(),
            last_params: None,
            results: Vec::new(),

            md_query: String::new(),
            md_last_query: None,
            md_results: Vec::new(),
            md_history: katana_core::search::SearchHistory::default(),
        }
    }
}
