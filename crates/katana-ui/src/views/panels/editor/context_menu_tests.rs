use super::context_menu::EditorContextMenu;

#[test]
fn markdown_format_is_offered_only_for_editable_markdown_documents() {
    assert!(EditorContextMenu::should_offer_markdown_format(
        std::path::Path::new("/workspace/README.md"),
        true
    ));
    assert!(EditorContextMenu::should_offer_markdown_format(
        std::path::Path::new("/workspace/README.markdown"),
        true
    ));
    assert!(!EditorContextMenu::should_offer_markdown_format(
        std::path::Path::new("/workspace/README.txt"),
        true
    ));
    assert!(!EditorContextMenu::should_offer_markdown_format(
        std::path::Path::new("/workspace/README.md"),
        false
    ));
}
