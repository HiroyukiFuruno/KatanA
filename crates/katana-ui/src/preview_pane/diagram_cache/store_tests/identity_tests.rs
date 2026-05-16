use super::super::content::DiagramCacheIdentityService;
use katana_core::markdown::DiagramKind;
use std::path::Path;

#[test]
fn document_path_separates_same_diagram_content() {
    let first = DiagramCacheIdentityService::build(
        Path::new("/tmp/first.md"),
        &DiagramKind::Mermaid,
        "graph TD;A-->B",
    );
    let second = DiagramCacheIdentityService::build(
        Path::new("/tmp/second.md"),
        &DiagramKind::Mermaid,
        "graph TD;A-->B",
    );

    assert_ne!(first.document_dir_name, second.document_dir_name);
    assert_eq!(first.content_checksum, second.content_checksum);
}
