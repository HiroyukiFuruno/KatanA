use crate::shell::KatanaApp;

pub(crate) trait DocSearchRefresh {
    fn refresh_doc_search_matches(&mut self, content: &str);
}

impl DocSearchRefresh for KatanaApp {
    fn refresh_doc_search_matches(&mut self, content: &str) {
        let query = &self.state.search.doc_search.query;
        self.state.search.doc_search_matches = DocSearchOps::compute_matches(query, content);
        self.state.search.doc_search_active_index = 0;
    }
}

/* WHY: Result of a search navigation action (next/prev). */
pub(crate) struct SearchNavResult {
    pub new_active_index: usize,
    pub scroll_to_line: Option<usize>,
}

/* WHY: Stateless operations for document-level search logic. */
pub(crate) struct DocSearchOps;

impl DocSearchOps {
    /* WHY: Compute character-based match ranges from a query and content string. */
    /* WHY: Returns `Vec<Range<usize>>` where indices are character offsets (not byte offsets). */
    /* WHY: This implementation is Markdown-aware: it only searches within parts of the document that are actually rendered as text or code, skipping URLs, hidden tag attributes, and metadata. */
    pub fn compute_matches(query: &str, content: &str) -> Vec<std::ops::Range<usize>> {
        let mut matches = Vec::new();
        if query.is_empty() {
            return matches;
        }

        let Ok(re) = regex::RegexBuilder::new(&regex::escape(query))
            .case_insensitive(true)
            .build()
        else {
            return matches;
        };

        use pulldown_cmark::{Event, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(content, options).into_offset_iter();

        for (event, range) in parser {
            match event {
                /* WHY: We only search for content text, inline code, or fenced code text. */
                /* WHY: We skip Event::Html as it usually contains non-rendered tags/attributes. */
                Event::Text(_) | Event::Code(_) => {
                    let snippet = &content[range.clone()];
                    for mat in re.find_iter(snippet) {
                        let global_byte_start = range.start + mat.start();
                        let global_byte_end = range.start + mat.end();

                        /* WHY: Convert byte offsets to character offsets for the editor highlight system. */
                        let char_start = content[..global_byte_start].chars().count();
                        let char_end = char_start
                            + content[global_byte_start..global_byte_end].chars().count();

                        matches.push(char_start..char_end);
                    }
                }
                _ => {}
            }
        }

        /* WHY: Standardize output: unique and sorted. */
        matches.sort_by_key(|r| r.start);
        matches.dedup();

        matches
    }

    /* WHY: Compute the result of navigating to the next search match. */
    pub fn navigate_next(
        matches: &[std::ops::Range<usize>],
        current_index: usize,
        buffer: &str,
    ) -> Option<SearchNavResult> {
        if matches.is_empty() {
            return None;
        }
        let len = matches.len();
        let new_index = (current_index + 1) % len;
        let r = &matches[new_index];
        let line = crate::views::panels::editor::types::EditorLogicOps::char_index_to_line(
            buffer, r.start,
        );
        Some(SearchNavResult {
            new_active_index: new_index,
            scroll_to_line: Some(line),
        })
    }

    /* WHY: Compute the result of navigating to the previous search match. */
    pub fn navigate_prev(
        matches: &[std::ops::Range<usize>],
        current_index: usize,
        buffer: &str,
    ) -> Option<SearchNavResult> {
        if matches.is_empty() {
            return None;
        }
        let len = matches.len();
        let new_index = (current_index + len - 1) % len;
        let r = &matches[new_index];
        let line = crate::views::panels::editor::types::EditorLogicOps::char_index_to_line(
            buffer, r.start,
        );
        Some(SearchNavResult {
            new_active_index: new_index,
            scroll_to_line: Some(line),
        })
    }
}

#[cfg(test)]
include!("doc_search_tests.rs");
