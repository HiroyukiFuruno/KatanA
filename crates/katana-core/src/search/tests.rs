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
    assert!(WorkspaceSearchOps::search_workspace(&ws, "", false, false, false, 10).is_empty());
}

#[test]
fn test_search_workspace_options() {
    let dir = tempdir().unwrap();
    let md_path = dir.path().join("options.md");
    let mut md = File::create(&md_path).unwrap();
    writeln!(md, "Apple banana cherry").unwrap();
    writeln!(md, "apple Pie").unwrap();

    let ws = crate::Workspace::new(
        dir.path().to_path_buf(),
        vec![crate::workspace::TreeEntry::File {
            path: md_path.clone(),
        }],
    );

    /* WHY: Default: case-insensitive */
    let res = WorkspaceSearchOps::search_workspace(&ws, "apple", false, false, false, 10);
    assert_eq!(res.len(), 2);

    /* WHY: Match case: only lowercase */
    let res_case = WorkspaceSearchOps::search_workspace(&ws, "apple", true, false, false, 10);
    assert_eq!(res_case.len(), 1);

    /* WHY: Match word: full word */
    let res_word = WorkspaceSearchOps::search_workspace(&ws, "apple", false, true, false, 10);
    assert_eq!(res_word.len(), 2);

    /* WHY: Use regex: with word boundaries */
    let res_re = WorkspaceSearchOps::search_workspace(&ws, "A.*e", false, false, true, 10);
    assert_eq!(res_re.len(), 2);

    /* WHY: Use regex + match word */
    let res_re_word = WorkspaceSearchOps::search_workspace(&ws, "Pie", false, true, true, 10);
    assert_eq!(res_re_word.len(), 1);
}

#[test]
fn test_search_workspace() {
    let dir = tempdir().unwrap();
    let md1_path = dir.path().join("a.md");
    let mut md1 = File::create(&md1_path).unwrap();
    writeln!(md1, "match").unwrap();
    writeln!(md1, "match").unwrap();

    let md2_path = dir.path().join("b.md");
    let mut md2 = File::create(&md2_path).unwrap();
    writeln!(md2, "match").unwrap();

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

    /* WHY: 1. Test line-loop break (within a.md, after first match) */
    let results = WorkspaceSearchOps::search_workspace(&ws, "match", false, false, false, 1);
    assert_eq!(results.len(), 1);

    /* WHY: 2. Test file-loop break (between a.md and b.md) */
    let results2 = WorkspaceSearchOps::search_workspace(&ws, "match", false, false, false, 2);
    assert_eq!(results2.len(), 2);
}

#[test]
fn test_search_noise_reduction() {
    let dir = tempdir().unwrap();
    let md_path = dir.path().join("noise.md");
    let mut md = File::create(&md_path).unwrap();
    writeln!(md, "#[allow(dead_code)]").unwrap();
    writeln!(md, "actual content").unwrap();

    let ws = crate::Workspace::new(
        dir.path().to_path_buf(),
        vec![crate::workspace::TreeEntry::File {
            path: md_path.clone(),
        }],
    );

    let results = WorkspaceSearchOps::search_workspace(&ws, "dead", false, false, false, 10);
    assert_eq!(results.len(), 0);

    let results_content =
        WorkspaceSearchOps::search_workspace(&ws, "content", false, false, false, 10);
    assert_eq!(results_content.len(), 1);
}
