use super::super::store::DiagramRenderCacheCoordinator;
use super::helpers::{count_files_with_extension, test_cache};
use katana_core::markdown::DiagramKind;
use std::path::Path;

#[test]
fn cached_svg_is_stored_as_svg_file_without_manifest() {
    let (_tmp, cache) = test_cache();
    let document_path = Path::new("/tmp/doc.md");
    let source = "graph TD;A-->B";

    DiagramRenderCacheCoordinator::store_svg(
        &cache,
        document_path,
        &DiagramKind::Mermaid,
        source,
        "<svg width=\"10\" height=\"10\"></svg>",
    );

    let cached = DiagramRenderCacheCoordinator::cached_svg(
        &cache,
        document_path,
        &DiagramKind::Mermaid,
        source,
    );
    let diagram_root = cache
        .diagram_cache_root()
        .expect("diagram root should be available");

    assert_eq!(
        cached.as_deref(),
        Some("<svg width=\"10\" height=\"10\"></svg>")
    );
    assert!(diagram_root.exists());
    assert_eq!(count_files_with_extension(&diagram_root, "svg"), 1);
    assert_eq!(count_files_with_extension(&diagram_root, "json"), 0);
}
