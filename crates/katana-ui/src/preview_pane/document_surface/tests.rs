use super::source::DocumentSurfaceSource;
use super::worker::DocumentWorkerCommand;
use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::{
    DocumentGridCommand, DocumentSurfaceCommand, DocumentViewerCommand, DocumentViewport,
};

#[test]
fn remote_source_keeps_final_url_and_validated_format() {
    let mut source = DocumentSurfaceSource::remote(
        "https://example.test/report.pdf".to_owned(),
        Some("application/pdf"),
        b"%PDF-1.7".to_vec(),
    )
    .expect("valid PDF source");

    assert_eq!(source.uri, "https://example.test/report.pdf");
    assert_eq!(source.mime, "application/pdf");
    assert_eq!(source.format, BinaryDocumentFormat::Pdf);
    assert_eq!(source, source.descriptor());
    assert_eq!(
        b"%PDF-1.7".as_slice(),
        source.take_bytes().expect("source bytes").as_slice()
    );
    assert!(source.take_bytes().is_err());
}

#[test]
fn local_source_keeps_canonical_identity_revision_and_bytes() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("report.PDF");
    std::fs::write(&path, b"%PDF-1.7").expect("PDF fixture");

    let mut source = DocumentSurfaceSource::local(&path).expect("local PDF source");

    assert_eq!(source.format, BinaryDocumentFormat::Pdf);
    assert_eq!(source.mime, BinaryDocumentFormat::Pdf.mime());
    assert_eq!(
        source.uri,
        url::Url::from_file_path(path.canonicalize().expect("canonical path"))
            .expect("file URL")
            .to_string()
    );
    assert!(source.revision.starts_with("sha256:"));
    assert_eq!(source.take_bytes().expect("source bytes"), b"%PDF-1.7");
}

#[test]
fn local_source_rejects_a_missing_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let missing = directory.path().join("missing.pdf");
    assert_eq!(
        DocumentSurfaceSource::local(&missing)
            .expect_err("missing document")
            .operation,
        "canonicalize"
    );
}

#[test]
fn local_source_rejects_an_unsupported_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let unsupported = directory.path().join("notes.txt");
    std::fs::write(&unsupported, b"plain text").expect("text fixture");
    assert_eq!(
        DocumentSurfaceSource::local(&unsupported)
            .expect_err("unsupported document")
            .operation,
        "classify"
    );
}

#[test]
fn local_source_rejects_a_signature_mismatch() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let malformed = directory.path().join("report.pdf");
    std::fs::write(&malformed, b"not a PDF").expect("malformed fixture");
    assert_eq!(
        DocumentSurfaceSource::local(&malformed)
            .expect_err("signature mismatch")
            .operation,
        "validate"
    );
}

#[test]
fn local_reader_stops_at_the_configured_limit() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("oversized.pdf");
    std::fs::write(&path, b"12345").expect("fixture");

    let failure = super::source::read_bounded_with_limit(&path, Some(BinaryDocumentFormat::Pdf), 4)
        .expect_err("oversized source");

    assert_eq!("validate size", failure.operation);
    assert!(failure.cause.contains("5 bytes; limit is 4 bytes"));
}

#[test]
fn remote_source_rejects_metadata_and_signature_mismatches() {
    let uri = "https://example.test/downloads/report.pdf?revision=4";
    let failure = DocumentSurfaceSource::remote(
        uri.to_owned(),
        Some("application/pdf"),
        b"not-a-pdf".to_vec(),
    );

    assert_eq!(uri, failure.expect_err("signature mismatch").document);
}

#[test]
fn bounded_queue_preserves_navigation_and_coalesces_continuous_state() {
    let mut pending = super::render_support::PendingDocumentCommands::default();
    let navigation = DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next);
    let previous = DocumentWorkerCommand::Viewer(DocumentViewerCommand::Previous);
    let resize = DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Resize(
        DocumentViewport::new(800, 600),
    ));
    pending.push(navigation);
    pending.push(resize);
    pending.push(previous);
    assert_eq!(Some(navigation), pending.take_next());
    assert_eq!(Some(previous), pending.take_next());

    let latest_resize = DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Resize(
        DocumentViewport::new(1_280, 720),
    ));
    pending.push(latest_resize);
    assert_eq!(Some(latest_resize), pending.take_next());

    let first_scroll = DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(
        DocumentGridCommand::ScrollTo { x: 10, y: 20 },
    ));
    let latest_scroll = DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(
        DocumentGridCommand::ScrollTo { x: 30, y: 40 },
    ));
    pending.push(first_scroll);
    pending.push(latest_scroll);
    assert_eq!(Some(latest_scroll), pending.take_next());

    let first_zoom = DocumentWorkerCommand::Viewer(DocumentViewerCommand::SetZoom(1.25));
    let latest_zoom = DocumentWorkerCommand::Viewer(DocumentViewerCommand::SetZoom(1.5));
    pending.push(first_zoom);
    pending.push(latest_zoom);
    assert_eq!(Some(latest_zoom), pending.take_next());

    let fit_page = DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(
        katana_document_viewer::DocumentFitMode::Page,
    ));
    let fit_width = DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(
        katana_document_viewer::DocumentFitMode::Width,
    ));
    pending.push(fit_page);
    pending.push(fit_width);
    assert_eq!(Some(fit_width), pending.take_next());
    assert!(pending.is_empty());

    pending.push(navigation);
    pending.push(resize);
    pending.clear();
    assert!(pending.is_empty());
}

#[test]
fn bounded_queue_keeps_the_most_recent_navigation_commands() {
    let mut pending = super::render_support::PendingDocumentCommands::default();
    for _ in 0..20 {
        pending.push(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next));
    }

    let mut count = 0;
    while pending.take_next().is_some() {
        count += 1;
    }
    assert_eq!(count, 16);
    assert!(pending.is_empty());
}

#[test]
fn document_surface_sends_only_changed_viewports() {
    let mut current = None;
    let initial = DocumentViewport::new(800, 600);
    let resized = DocumentViewport::new(1_280, 720);

    assert!(super::render::replace_viewport_if_changed(
        &mut current,
        initial
    ));
    assert!(!super::render::replace_viewport_if_changed(
        &mut current,
        initial
    ));
    assert!(super::render::replace_viewport_if_changed(
        &mut current,
        resized
    ));
    assert_eq!(Some(resized), current);
}

#[test]
fn failure_details_preserve_every_trace_field() {
    let failure = super::types::DocumentFailure::new(
        super::types::DocumentFailureLayer::KdvWorker,
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
    use super::types::DocumentFailureLayer;

    assert_eq!("source intake", DocumentFailureLayer::SourceIntake.label());
    assert_eq!("KDV worker", DocumentFailureLayer::KdvWorker.label());
    assert_eq!(
        "KDV document surface",
        DocumentFailureLayer::KdvSurface.label()
    );
    assert_eq!("KatanA host", DocumentFailureLayer::KatanaHost.label());
}
