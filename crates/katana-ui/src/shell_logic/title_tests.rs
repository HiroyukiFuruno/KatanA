use super::ShellLogicOps;
use std::path::Path;

#[test]
fn workspace_window_title_uses_workspace_name() {
    let title = ShellLogicOps::format_workspace_window_title(Some(Path::new(
        "/Users/example/workspace/katana",
    )));

    assert_eq!(title, "katana - KatanA");
}

#[test]
fn workspace_window_title_without_workspace_uses_app_name() {
    let title = ShellLogicOps::format_workspace_window_title(None);

    assert_eq!(title, "KatanA");
}

#[test]
fn app_title_keeps_active_document_context() {
    let title = ShellLogicOps::format_document_title(
        "sample.ja.md",
        "assets/fixtures/sample.ja.md",
        "Release Notes",
    );

    assert_eq!(
        title,
        "sample.ja.md (assets/fixtures/sample.ja.md) - KatanA"
    );
}
