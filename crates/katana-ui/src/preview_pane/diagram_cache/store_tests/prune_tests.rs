use super::super::store::DiagramRenderCacheCoordinator;
use super::helpers::{count_files_with_extension, svg_file_names, test_cache};
use katana_core::markdown::DiagramKind;
use std::path::Path;

#[test]
fn prune_removes_deleted_middle_diagram_without_order_key() {
    let (_tmp, cache) = test_cache();
    let document_path = Path::new("/tmp/doc.md");
    let sources = ["A", "B", "C", "D", "E", "F", "G"];

    for source in sources {
        DiagramRenderCacheCoordinator::store_svg(
            &cache,
            document_path,
            &DiagramKind::Mermaid,
            source,
            &format!("<svg><text>{source}</text></svg>"),
        );
    }

    let active_sources = ["A", "B", "D", "E", "F", "G"];
    DiagramRenderCacheCoordinator::prune_document(
        &cache,
        document_path,
        &active_sources
            .iter()
            .map(|source| (DiagramKind::Mermaid, (*source).to_string()))
            .collect::<Vec<_>>(),
    );

    assert!(
        DiagramRenderCacheCoordinator::cached_svg(
            &cache,
            document_path,
            &DiagramKind::Mermaid,
            "C",
        )
        .is_none()
    );
    assert_eq!(
        count_files_with_extension(
            &cache
                .diagram_cache_root()
                .expect("diagram root should be available"),
            "svg",
        ),
        6
    );
}

#[test]
fn reordered_diagrams_reuse_same_svg_cache_files() {
    let (_tmp, cache) = test_cache();
    let document_path = Path::new("/tmp/doc.md");

    for source in ["A", "B", "C"] {
        DiagramRenderCacheCoordinator::store_svg(
            &cache,
            document_path,
            &DiagramKind::Mermaid,
            source,
            &format!("<svg><text>{source}</text></svg>"),
        );
    }
    let root = cache
        .diagram_cache_root()
        .expect("diagram root should be available");
    let before = svg_file_names(&root);

    DiagramRenderCacheCoordinator::prune_document(
        &cache,
        document_path,
        &[
            (DiagramKind::Mermaid, "C".to_string()),
            (DiagramKind::Mermaid, "A".to_string()),
            (DiagramKind::Mermaid, "B".to_string()),
        ],
    );

    assert_eq!(before, svg_file_names(&root));
}

#[test]
fn identical_diagrams_in_same_document_share_one_svg_file() {
    let (_tmp, cache) = test_cache();
    let document_path = Path::new("/tmp/doc.md");
    let source = "graph TD;A-->B";

    DiagramRenderCacheCoordinator::store_svg(
        &cache,
        document_path,
        &DiagramKind::Mermaid,
        source,
        "<svg><text>A</text></svg>",
    );
    DiagramRenderCacheCoordinator::store_svg(
        &cache,
        document_path,
        &DiagramKind::Mermaid,
        source,
        "<svg><text>A</text></svg>",
    );

    let root = cache
        .diagram_cache_root()
        .expect("diagram root should be available");
    assert_eq!(count_files_with_extension(&root, "svg"), 1);
}
