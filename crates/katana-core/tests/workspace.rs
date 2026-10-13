use katana_core::workspace::*;
use std::path::PathBuf;

#[test]
fn standard_workspace_extensions_include_read_only_canvas() {
    assert!(TreeEntry::standard_visible_extensions().contains(&"canvas"));
}

#[test]
fn canvas_classification_preserves_other_formats_and_directory_identity() {
    for name in ["graph.canvas", "graph.CANVAS"] {
        let entry = TreeEntry::File {
            path: PathBuf::from(name),
        };
        assert!(entry.is_canvas());
        assert!(!entry.is_document());
        assert!(!entry.is_image());
        assert!(!entry.is_markdown());
    }
    for name in [
        "graph.canvas.md",
        "graphcanvas",
        "graph.json",
        "graph.canvasx",
    ] {
        assert!(!TreeEntry::path_is_canvas(std::path::Path::new(name)));
    }
    let directory = TreeEntry::Directory {
        path: PathBuf::from("graph.canvas"),
        children: vec![],
    };
    assert!(!directory.is_canvas());
    assert_eq!(TreeEntry::canvas_extensions(), &["canvas"]);
}

#[test]
fn workspace_name_is_root_basename() {
    let ws = Workspace::new("/home/user/myproject", vec![]);
    assert_eq!(ws.name(), Some("myproject"));
}

#[test]
fn tree_entry_identifies_markdown_files() {
    let md = TreeEntry::File {
        path: PathBuf::from("docs/spec.md"),
    };
    let rs = TreeEntry::File {
        path: PathBuf::from("src/main.rs"),
    };
    assert!(md.is_markdown());
    assert!(!rs.is_markdown());
}

#[test]
fn directory_entry_is_not_markdown() {
    let dir = TreeEntry::Directory {
        path: PathBuf::from("docs"),
        children: vec![],
    };
    assert!(!dir.is_markdown());
    assert!(!dir.is_file());
}

#[test]
fn tree_entry_path_and_name_for_directory() {
    let path = PathBuf::from("docs/chapter1");
    let entry = TreeEntry::Directory {
        path: path.clone(),
        children: vec![],
    };
    assert_eq!(entry.path(), path.as_path());
    assert_eq!(entry.name(), Some("chapter1"));
}

#[test]
fn tree_entry_path_for_file() {
    let path = PathBuf::from("docs/spec.md");
    let entry = TreeEntry::File { path: path.clone() };
    assert_eq!(entry.path(), path.as_path());
}

#[test]
fn workspace_error_unreadable_root_message() {
    let err = WorkspaceError::unreadable_root(
        PathBuf::from("/some/path"),
        std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    );
    let msg = err.to_string();
    assert!(msg.contains("/some/path"));
}

#[test]
fn workspace_error_no_workspace_message() {
    let err = WorkspaceError::NoWorkspace;
    let msg = err.to_string();
    assert!(msg.contains("workspace") || msg.contains("Workspace"));
}

#[test]
fn workspace_name_none_for_filesystem_root() {
    let ws = Workspace::new("/", vec![]);
    assert_eq!(ws.name(), None);
}

#[test]
fn is_markdown_returns_false_for_no_extension() {
    let entry = TreeEntry::File {
        path: PathBuf::from("Justfile"),
    };
    assert!(!entry.is_markdown());
}
