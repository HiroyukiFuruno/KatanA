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
    pub last_params: Option<(SearchParams, String, String, Option<PathBuf>)>,
    pub results: Vec<PathBuf>,

    /* WHY: Markdown content search state */
    pub md_search: SearchParams,
    pub md_last_params: Option<(SearchParams, Option<PathBuf>)>,
    pub md_results: Vec<katana_core::search::SearchResult>,
    pub md_history: katana_core::search::SearchHistory,
    pub md_history_cursor: Option<usize>,
    pub md_history_draft: String,

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
                use_regex: false,
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
            md_history_cursor: None,
            md_history_draft: String::new(),

            doc_search_open: false,
            doc_search: SearchParams::default(),
            doc_search_matches: Vec::new(),
            doc_search_active_index: 0,
        }
    }

    pub fn clear_workspace_scoped_results(&mut self) {
        self.filter_cache = None;
        self.last_params = None;
        self.results.clear();
        self.md_last_params = None;
        self.md_results.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_workspace_scoped_results_preserves_queries_and_history() {
        let mut state = SearchState::new();
        state.filter_cache = Some((SearchParams::default(), HashSet::from([PathBuf::from("a")])));
        state.file_search.query = "file".to_string();
        state.last_params = Some((
            state.file_search.clone(),
            String::new(),
            String::new(),
            Some(PathBuf::from("/workspace/a")),
        ));
        state.results = vec![PathBuf::from("/workspace/a/file.md")];
        state.md_search.query = "body".to_string();
        state.md_last_params = Some((state.md_search.clone(), Some(PathBuf::from("/workspace/a"))));
        state.md_results = vec![katana_core::search::SearchResult {
            file_path: PathBuf::from("/workspace/a/file.md"),
            line_number: 0,
            start_col: 0,
            end_col: 4,
            snippet: "body".to_string(),
        }];
        state.md_history.push_term("body".to_string(), 10);

        state.clear_workspace_scoped_results();

        assert!(state.filter_cache.is_none());
        assert!(state.last_params.is_none());
        assert!(state.results.is_empty());
        assert!(state.md_last_params.is_none());
        assert!(state.md_results.is_empty());
        assert_eq!(state.file_search.query, "file");
        assert_eq!(state.md_search.query, "body");
        assert_eq!(state.md_history.recent_terms, vec!["body"]);
    }
}
