use katana_core::workspace::{TreeEntry, Workspace};
use std::path::PathBuf;

#[test]
fn workspace_keeps_its_original_public_struct_literal_shape() {
    let workspace = Workspace {
        root: PathBuf::from("/workspace"),
        tree: vec![TreeEntry::File {
            path: PathBuf::from("/workspace/readme.md"),
        }],
    };

    assert_eq!(workspace.root, PathBuf::from("/workspace"));
    assert_eq!(workspace.tree.len(), 1);
}
