use super::*;
use std::path::PathBuf;

#[test]
fn test_workspace_collection() {
    let root = PathBuf::from("/root");
    let file1 = root.join("a.md");
    let sub = root.join("sub");
    let file2 = sub.join("b.md");

    let workspace = Workspace::new(
        root.clone(),
        vec![
            TreeEntry::File {
                path: file1.clone(),
            },
            TreeEntry::Directory {
                path: sub.clone(),
                children: vec![TreeEntry::File {
                    path: file2.clone(),
                }],
            },
        ],
    );

    let mds = workspace.collect_all_markdown_file_paths();
    assert_eq!(mds.len(), 2);
    assert!(mds.contains(&file1));
    assert!(mds.contains(&file2));

    let dirs = workspace.collect_all_directory_paths();
    assert_eq!(dirs.len(), 1);
    assert!(dirs.contains(&sub));
}

#[test]
fn test_image_extension_detection() {
    let image = TreeEntry::File {
        path: PathBuf::from("/root/asset/img/example.PNG"),
    };
    let markdown = TreeEntry::File {
        path: PathBuf::from("/root/readme.md"),
    };
    let directory = TreeEntry::Directory {
        path: PathBuf::from("/root/asset"),
        children: Vec::new(),
    };

    assert!(image.is_image());
    assert!(!markdown.is_image());
    assert!(!directory.is_image());
    assert!(TreeEntry::image_extensions().contains(&"png"));
}
