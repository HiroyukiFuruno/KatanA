use katana_core::document_source::BinaryDocumentFormat;

use super::types::{DocumentFailure, DocumentFailureLayer};

#[test]
fn failure_details_preserve_every_trace_field() {
    let failure = DocumentFailure::new(
        DocumentFailureLayer::KdvWorker,
        "render",
        "https://example.test/report.pdf?revision=4".to_owned(),
        Some(BinaryDocumentFormat::Pdf),
        "isolated worker exited with status 2",
    );

    assert_eq!(
        "KDV worker could not display this pdf document",
        failure.summary()
    );
    let details = failure.details();
    for expected in [
        "Layer: KDV worker",
        "Operation: render",
        "Format: pdf",
        "Document: https://example.test/report.pdf?revision=4",
        "Cause: isolated worker exited with status 2",
    ] {
        assert!(details.contains(expected), "missing diagnostic: {expected}");
    }
}

#[test]
fn failure_layers_keep_the_kdv_surface_boundary_explicit() {
    assert_eq!("source intake", DocumentFailureLayer::SourceIntake.label());
    assert_eq!("KDV worker", DocumentFailureLayer::KdvWorker.label());
    assert_eq!(
        "KDV document surface",
        DocumentFailureLayer::KdvSurface.label()
    );
    assert_eq!("KatanA host", DocumentFailureLayer::KatanaHost.label());
}
