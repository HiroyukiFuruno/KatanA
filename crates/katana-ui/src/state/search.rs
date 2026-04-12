use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SearchTab {
    FileName,
    MarkdownContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchParams {
    pub query: String,
    pub match_case: bool,
    pub match_word: bool,
    pub use_regex: bool,
}

pub struct SearchState {
    pub filter_enabled: bool,
    pub filter: SearchParams,
    pub filter_cache: Option<(SearchParams, HashSet<PathBuf>)>,

    /* WHY: UI state */
    pub active_tab: SearchTab,
    pub focus_requested: bool,

    /* WHY: File name search state */
    pub file_search: SearchParams,
    pub include_pattern: String,
    pub exclude_pattern: String,
    pub last_params: Option<(SearchParams, String, String)>,
    pub results: Vec<PathBuf>,

    /* WHY: Markdown content search state */
    pub md_search: SearchParams,
    pub md_last_params: Option<SearchParams>,
    pub md_results: Vec<katana_core::search::SearchResult>,
    pub md_history: katana_core::search::SearchHistory,

    /* WHY: Document search state (Command+F) */
    pub doc_search_open: bool,
    pub doc_search: SearchParams,
    pub doc_search_matches: Vec<std::ops::Range<usize>>,
    pub doc_search_active_index: usize,
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
            filter: SearchParams {
                query: String::new(),
                match_case: false,
                match_word: false,
                use_regex: true,
            },
            filter_cache: None,

            active_tab: SearchTab::FileName,
            focus_requested: false,

            file_search: SearchParams::default(),
            include_pattern: String::new(),
            exclude_pattern: String::new(),
            last_params: None,
            results: Vec::new(),

            md_search: SearchParams::default(),
            md_last_params: None,
            md_results: Vec::new(),
            md_history: katana_core::search::SearchHistory::default(),

            doc_search_open: false,
            doc_search: SearchParams::default(),
            doc_search_matches: Vec::new(),
            doc_search_active_index: 0,
        }
    }
}
