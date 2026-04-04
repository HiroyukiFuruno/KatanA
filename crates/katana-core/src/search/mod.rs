mod types;
pub use types::*;

pub struct WorkspaceSearchOps;

impl WorkspaceSearchOps {
    pub fn search_workspace(
        workspace: &crate::Workspace,
        query: &str,
        limit: usize,
    ) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
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

            for (line_idx, line) in content.lines().enumerate() {
                if results.len() >= limit {
                    break;
                }

                let line_lower = line.to_lowercase();
                for (byte_offset, _) in line_lower.match_indices(&query_lower) {
                    if results.len() >= limit {
                        break;
                    }

                    let snippet = line.to_string();
                    results.push(SearchResult {
                        file_path: file_path.clone(),
                        line_number: line_idx,
                        start_col: byte_offset,
                        end_col: byte_offset + query_lower.len(),
                        snippet,
                    });
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests;
