use std::path::PathBuf;

/// Defines the contract for Markdown Content Search.
///
/// # Search Contract
///
/// 1. **Result Structure (`1 result = 1 matched location`)**:
///    Unlike file search which returns one entry per file, this search returns one entry per *match*.
///    If a file contains 5 matches, it yields 5 separate `SearchResult` items.
/// 2. **Search Scope**:
///    Only `.md` and `.mdx` files within the active `Workspace` are searched.
///    Hidden files, directories, and non-markdown files are excluded.
/// 3. **Ranking/Ordering Rule**:
///    Results MUST be ordered. The recommended order is by file path lexicographically,
///    and then by line number ascending. This ensures predictable navigation.
/// 4. **Source Position Representation**:
///    Matches include exact `line` (0-indexed or 1-indexed, consistent with editor) and byte offsets.
/// 5. **Usage Context**:
///    This is explicitly separate from `WorkspaceFileSearch`. It serves content discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// The absolute path to the matched markdown file.
    pub file_path: PathBuf,

    /// The 0-based index of the line where the match occurs.
    pub line_number: usize,

    /// The 0-based byte offset within the line where the match starts.
    pub start_col: usize,

    /// The 0-based byte offset within the line where the match ends.
    pub end_col: usize,

    /// A short snippet of text surrounding the match for UI display.
    pub snippet: String,
}

/// A collection of recent search terms, scoped to the user profile (not the workspace).
#[derive(Debug, Clone, Default)]
pub struct SearchHistory {
    /// The history of recent search queries, ordered from newest to oldest.
    pub recent_terms: Vec<String>,
}

impl SearchHistory {
    /// Adds a term to history, bounded to a specific maximum capacity.
    /// If the term already exists, it is moved to the top.
    pub fn push_term(&mut self, term: String, max_items: usize) {
        if term.trim().is_empty() {
            return;
        }
        self.recent_terms.retain(|t| t != &term);
        self.recent_terms.insert(0, term);
        self.recent_terms.truncate(max_items);
    }

    /// Clears all recent search history.
    pub fn clear(&mut self) {
        self.recent_terms.clear();
    }
}

/// Searches the workspace's markdown files for the given `query`.
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

    // Iterate lexicographically to satisfy the ranking constraint in typical cases.
    // (Though paths from `collect_all_markdown_file_paths` might not be perfectly sorted,
    // it's usually deterministic based on tree structure. We sort to be certain.)
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
            // A simple substring search.
            // In a more complex implementation, we might find multiple matches per line.
            // For now, we find the first match per line for simplicity, or we can use `match_indices`.
            for (byte_offset, _) in line_lower.match_indices(&query_lower) {
                if results.len() >= limit {
                    break;
                }

                let snippet = line.to_string(); // we take the whole line as snippet for now
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_search_history() {
        let mut history = SearchHistory::default();
        history.push_term("apple".to_string(), 3);
        assert_eq!(history.recent_terms, vec!["apple"]);

        history.push_term("banana".to_string(), 3);
        history.push_term("cherry".to_string(), 3);
        history.push_term("date".to_string(), 3);
        // "apple" is pushed out
        assert_eq!(history.recent_terms, vec!["date", "cherry", "banana"]);

        // empty term is ignored
        history.push_term("   ".to_string(), 3);
        assert_eq!(history.recent_terms, vec!["date", "cherry", "banana"]);

        // existing term is moved to front
        history.push_term("banana".to_string(), 3);
        assert_eq!(history.recent_terms, vec!["banana", "date", "cherry"]);

        history.clear();
        assert!(history.recent_terms.is_empty());
    }

    #[test]
    fn test_search_workspace_empty_query() {
        let ws = crate::Workspace::new(PathBuf::from("/"), vec![]);
        assert!(search_workspace(&ws, "", 10).is_empty());
    }

    #[test]
    fn test_search_workspace() {
        let dir = tempdir().unwrap();
        let md1_path = dir.path().join("a.md");
        let mut md1 = File::create(&md1_path).unwrap();
        writeln!(md1, "Hello world").unwrap();
        writeln!(md1, "foo Bar baz").unwrap();

        let md2_path = dir.path().join("b.md");
        let mut md2 = File::create(&md2_path).unwrap();
        writeln!(md2, "BAR test").unwrap();
        writeln!(md2, "nothing here").unwrap();

        let ws = crate::Workspace::new(
            dir.path().to_path_buf(),
            vec![
                crate::workspace::TreeEntry::File {
                    path: md1_path.clone(),
                },
                crate::workspace::TreeEntry::File {
                    path: md2_path.clone(),
                },
            ],
        );

        let results = search_workspace(&ws, "bar", 10);
        assert_eq!(results.len(), 2);

        // Files are ordered alphabetically: a.md then b.md
        let r1 = &results[0];
        assert_eq!(r1.file_path, md1_path);
        assert_eq!(r1.line_number, 1);
        assert_eq!(r1.start_col, 4);
        assert_eq!(r1.end_col, 7);
        assert_eq!(r1.snippet, "foo Bar baz");

        let r2 = &results[1];
        assert_eq!(r2.file_path, md2_path);
        assert_eq!(r2.line_number, 0);
        assert_eq!(r2.start_col, 0);
        assert_eq!(r2.end_col, 3);
        assert_eq!(r2.snippet, "BAR test");

        // Test limit
        let results_limited = search_workspace(&ws, "bar", 1);
        assert_eq!(results_limited.len(), 1);
        assert_eq!(results_limited[0].file_path, md1_path);

        // Test unreadable file handling (simulate by deleting a file)
        std::fs::remove_file(&md2_path).unwrap();
        let results_after_delete = search_workspace(&ws, "bar", 10);
        assert_eq!(results_after_delete.len(), 1);

        // Test limit breaking within lines (outer loop break)
        let md3_path = dir.path().join("c.md");
        let mut md3 = File::create(&md3_path).unwrap();
        writeln!(md3, "foo").unwrap();
        writeln!(md3, "foo").unwrap();

        // Test limit breaking within a single line (inner loop break)
        writeln!(md3, "foo foo").unwrap();

        let ws2 = crate::Workspace::new(
            dir.path().to_path_buf(),
            vec![crate::workspace::TreeEntry::File {
                path: md3_path.clone(),
            }],
        );

        // 1. Break between lines
        let results_line_break = search_workspace(&ws2, "foo", 1);
        assert_eq!(results_line_break.len(), 1);

        // 2. Break within the same line
        let results_inline_break = search_workspace(&ws2, "foo", 3);
        assert_eq!(results_inline_break.len(), 3);
    }
}
