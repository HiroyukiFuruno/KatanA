use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Default)]
pub struct SearchHistory {
    pub recent_terms: Vec<String>,
}

impl SearchHistory {
    pub fn push_term(&mut self, term: String, max_items: usize) {
        if term.trim().is_empty() {
            return;
        }
        self.recent_terms.retain(|t| t != &term);
        self.recent_terms.insert(0, term);
        self.recent_terms.truncate(max_items);
    }

    pub fn remove_term(&mut self, term: &str) {
        self.recent_terms.retain(|recent_term| recent_term != term);
    }

    pub fn clear(&mut self) {
        self.recent_terms.clear();
    }
}
