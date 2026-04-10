use super::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_search_history() {
    let mut history = SearchHistory::default();
    history.push_term("apple".to_string(), 3);
    assert_eq!(history.recent_terms, vec!["apple"]);

    history.push_term("banana".to_string(), 3);
    history.push_term("cherry".to_string(), 3);
    history.push_term("date".to_string(), 3);
    assert_eq!(history.recent_terms, vec!["date", "cherry", "banana"]);

    history.push_term("   ".to_string(), 3);
    assert_eq!(history.recent_terms, vec!["date", "cherry", "banana"]);

    history.push_term("banana".to_string(), 3);
    assert_eq!(history.recent_terms, vec!["banana", "date", "cherry"]);

    history.clear();
    assert!(history.recent_terms.is_empty());
}

#[test]
fn test_search_workspace_empty_query() {
    let ws = crate::Workspace::new(PathBuf::from("/"), vec![]);
    assert!(WorkspaceSearchOps::search_workspace(&ws, "", 10).is_empty());
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

    let results = WorkspaceSearchOps::search_workspace(&ws, "bar", 10);
    assert_eq!(results.len(), 2);

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

    let results_limited = WorkspaceSearchOps::search_workspace(&ws, "bar", 1);
    assert_eq!(results_limited.len(), 1);
    assert_eq!(results_limited[0].file_path, md1_path);

    std::fs::remove_file(&md2_path).unwrap();
    let results_after_delete = WorkspaceSearchOps::search_workspace(&ws, "bar", 10);
    assert_eq!(results_after_delete.len(), 1);

    let md3_path = dir.path().join("c.md");
    let mut md3 = File::create(&md3_path).unwrap();
    writeln!(md3, "foo").unwrap();
    writeln!(md3, "foo").unwrap();
    writeln!(md3, "foo foo").unwrap();

    let ws2 = crate::Workspace::new(
        dir.path().to_path_buf(),
        vec![crate::workspace::TreeEntry::File {
            path: md3_path.clone(),
        }],
    );

    let results_line_break = WorkspaceSearchOps::search_workspace(&ws2, "foo", 1);
    assert_eq!(results_line_break.len(), 1);

    let results_inline_break = WorkspaceSearchOps::search_workspace(&ws2, "foo", 3);
    assert_eq!(results_inline_break.len(), 3);
}

#[test]
fn test_search_noise_reduction() {
    let dir = tempdir().unwrap();
    let md_path = dir.path().join("noise.md");
    let mut md = File::create(&md_path).unwrap();
    writeln!(md, "#[allow(dead_code)]").unwrap();
    writeln!(md, "actual content").unwrap();
    writeln!(md, "   #[allow(unused)]").unwrap();
    writeln!(md, "some code // #[allow(none)]").unwrap();

    let ws = crate::Workspace::new(
        dir.path().to_path_buf(),
        vec![crate::workspace::TreeEntry::File {
            path: md_path.clone(),
        }],
    );

    /* WHY: Default search should skip #[allow(...)] lines */
    let results = WorkspaceSearchOps::search_workspace(&ws, "dead", 10);
    assert_eq!(results.len(), 0);

    let results_content = WorkspaceSearchOps::search_workspace(&ws, "content", 10);
    assert_eq!(results_content.len(), 1);
    assert_eq!(results_content[0].snippet, "actual content");

    /* WHY: Explicitly searching for 'allow' should show them */
    let results_allow = WorkspaceSearchOps::search_workspace(&ws, "allow", 10);
    assert_eq!(results_allow.len(), 3);

    /* WHY: 'unused' should be skipped by default */
    let results_unused = WorkspaceSearchOps::search_workspace(&ws, "unused", 10);
    assert_eq!(results_unused.len(), 0);
}
