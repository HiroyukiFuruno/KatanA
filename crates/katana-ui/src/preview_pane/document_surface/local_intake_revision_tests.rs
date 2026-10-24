use super::super::source::DocumentSurfaceSource;
use super::super::source_revision::{IntakeOutcome, SourceRevision};
use super::{LocalDocumentIntake, PreviewPane};

#[test]
fn manual_promotion_of_queued_unchanged_result_starts_forced_payload_read() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("manual.pdf");
    std::fs::write(&path, b"%PDF-1.7\nmanual").unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    let mut pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(known.clone())).unwrap();
    pending.force = true;
    let mut pane = PreviewPane::default();
    pane.finish_document_intake(pending, Ok(IntakeOutcome::Unchanged(known)));
    let restarted = pane.document_intake.as_ref().unwrap();
    assert!(restarted.force);
    assert!(restarted.request.known_revision.is_none());
}

fn loaded_pane(path: &std::path::Path) -> PreviewPane {
    std::fs::write(
        path,
        include_bytes!(
            "../../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        ),
    )
    .unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_source(DocumentSurfaceSource::local(path).unwrap(), false);
    pane
}

#[test]
fn unchanged_result_preserves_matching_generation_and_queued_recheck() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("same.pdf");
    let mut pane = loaded_pane(&path);
    let known = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    let pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(known.clone())).unwrap();
    pane.finish_document_intake(pending, Ok(IntakeOutcome::Unchanged(known.clone())));
    assert!(known.matches(pane.document_surface.as_ref()));
    assert!(pane.document_intake.is_none());
    let mut pending = LocalDocumentIntake::start_known(path, false, Some(known.clone())).unwrap();
    pending.refresh_requested = true;
    pane.finish_document_intake(pending, Ok(IntakeOutcome::Unchanged(known.clone())));
    assert_eq!(
        pane.document_intake
            .as_ref()
            .unwrap()
            .request
            .known_revision,
        Some(known)
    );
}

#[test]
fn queued_unchanged_result_cannot_replace_a_newer_surface_generation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("generation.pdf");
    let mut pane = loaded_pane(&path);
    let previous = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    let pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(previous.clone())).unwrap();
    pane.full_render_document_source(DocumentSurfaceSource::local(&path).unwrap(), true);
    let current = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    assert_ne!(previous, current);
    pane.finish_document_intake(pending, Ok(IntakeOutcome::Unchanged(previous)));
    assert!(current.matches(pane.document_surface.as_ref()));
    assert_eq!(
        pane.document_intake
            .as_ref()
            .unwrap()
            .request
            .known_revision,
        Some(current)
    );
}
