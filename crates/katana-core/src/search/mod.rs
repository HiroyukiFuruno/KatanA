mod types;
pub use types::*;

pub struct WorkspaceSearchOps;

impl WorkspaceSearchOps {
    pub fn search_workspace(
        workspace: &crate::Workspace,
        query: &str,
        match_case: bool,
        match_word: bool,
        use_regex: bool,
        limit: usize,
    ) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        let markdown_files = workspace.collect_all_markdown_file_paths();

        let mut sorted_files = markdown_files;
        sorted_files.sort();

        for file_path in sorted_files {
            if results.len() >= limit {
                break;
            }

            let Ok(content) = std::fs::read_to_string(&file_path) else {
                continue;
            };

            let (pattern, case_insensitive) = if use_regex {
                let p = if match_word {
                    format!(r"\b{}\b", query)
                } else {
                    query.to_string()
                };
                (p, !match_case)
            } else {
                let p = regex::escape(query);
                let p = if match_word { format!(r"\b{}\b", p) } else { p };
                (p, !match_case)
            };

            let Ok(re) = regex::RegexBuilder::new(&pattern)
                .case_insensitive(case_insensitive)
                .build()
            else {
                continue;
            };

            for (line_idx, line) in content.lines().enumerate() {
                if results.len() >= limit {
                    break;
                }

                /* WHY: Skip noise lines like #[allow(...)] unless the query itself contains 'allow'. */
                if line.contains("#[allow(") && !query.to_lowercase().contains("allow") {
                    continue;
                }

                let remaining = limit.saturating_sub(results.len());
                results.extend(re.find_iter(line).take(remaining).map(|m| SearchResult {
                    file_path: file_path.clone(),
                    line_number: line_idx,
                    start_col: m.start(),
                    end_col: m.end(),
                    snippet: line.to_string(),
                }));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests;
