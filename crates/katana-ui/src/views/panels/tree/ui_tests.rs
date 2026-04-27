use super::types::TreeContextMenu;
use katana_core::workspace::TreeEntry;
use std::path::PathBuf;

#[test]
fn markdown_format_is_offered_only_for_markdown_files() {
    let markdown = TreeEntry::File {
        path: PathBuf::from("/workspace/README.md"),
    };
    let markdown_long_ext = TreeEntry::File {
        path: PathBuf::from("/workspace/README.markdown"),
    };
    let text = TreeEntry::File {
        path: PathBuf::from("/workspace/README.txt"),
    };
    let directory = TreeEntry::Directory {
        path: PathBuf::from("/workspace/docs"),
        children: Vec::new(),
    };

    assert!(TreeContextMenu::should_offer_markdown_format(Some(
        &markdown
    )));
    assert!(TreeContextMenu::should_offer_markdown_format(Some(
        &markdown_long_ext
    )));
    assert!(!TreeContextMenu::should_offer_markdown_format(Some(&text)));
    assert!(!TreeContextMenu::should_offer_markdown_format(Some(
        &directory
    )));
    assert!(!TreeContextMenu::should_offer_markdown_format(None));
}

#[test]
fn directory_markdown_format_targets_selected_directory() {
    let dir = PathBuf::from("/workspace/docs");

    assert_eq!(
        TreeContextMenu::format_directory_markdown_action(&dir),
        crate::app_state::AppAction::FormatWorkspaceMarkdown(dir)
    );
}
