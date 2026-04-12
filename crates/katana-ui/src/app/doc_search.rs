use crate::shell::KatanaApp;

pub(crate) trait DocSearchRefresh {
    fn refresh_doc_search_matches(&mut self, content: &str);
}

impl DocSearchRefresh for KatanaApp {
    fn refresh_doc_search_matches(&mut self, content: &str) {
        let query = &self.state.search.doc_search.query;
        self.state.search.doc_search_matches.clear();
        self.state.search.doc_search_active_index = 0;
        if !query.is_empty()
            && let Ok(re) = regex::RegexBuilder::new(&regex::escape(query))
                .case_insensitive(true)
                .build()
        {
            let mut char_count = 0;
            let mut last_byte = 0;
            for mat in re.find_iter(content) {
                let mut start_b = mat.start();
                while start_b > 0 && !content.is_char_boundary(start_b) {
                    start_b -= 1;
                }
                let mut end_b = mat.end();
                while end_b < content.len() && !content.is_char_boundary(end_b) {
                    end_b += 1;
                }
                if start_b < last_byte {
                    start_b = last_byte;
                }
                if end_b < start_b {
                    end_b = start_b;
                }
                char_count += content[last_byte..start_b].chars().count();
                let char_start = char_count;
                let match_len = content[start_b..end_b].chars().count();
                let char_end = char_start + match_len;
                self.state
                    .search
                    .doc_search_matches
                    .push(char_start..char_end);
                char_count += match_len;
                last_byte = end_b;
            }
        }
    }
}
