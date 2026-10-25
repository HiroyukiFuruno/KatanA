use super::{DocumentReceiptObservation, HtmlLoadedSourceObservation};
use crate::preview_pane::{DocumentSurfaceSource, PreviewPane};
use crate::shell::resource_snapshot::UiResourceSnapshot;
use katana_core::document::{Document, DocumentOps};

#[test]
fn html_observation_never_loads_missing_or_unloaded_documents() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("not-on-disk.html");
    let unloaded = Document::new_empty(&path);
    assert!(HtmlLoadedSourceObservation::capture(None, 0).is_none());
    assert!(HtmlLoadedSourceObservation::capture(Some(&unloaded), 0).is_none());
    assert!(!path.exists());
    assert!(!unloaded.is_loaded);
    assert!(unloaded.buffer.is_empty());
}

#[test]
fn loaded_html_observation_keeps_actual_dirty_and_imported_revisions_separate() {
    let original = "<p>loaded</p>";
    let unsaved = "<p>unsaved</p>";
    let mut document = Document::new("local.html", original);
    let input_hash = DocumentOps::compute_hash(original);
    document.update_buffer(unsaved);
    let observed = HtmlLoadedSourceObservation::capture(Some(&document), input_hash).unwrap();
    assert_eq!(observed.preview_input_fnv1a64, format!("{input_hash:016x}"));
    assert_eq!(
        observed.last_imported_disk_fnv1a64,
        Some(format!("{input_hash:016x}"))
    );
    assert!(observed.dirty);
    assert_eq!(observed.document_revision, document.revision);
    assert_eq!(observed.saved_revision, document.saved_revision);
    assert_ne!(observed.document_revision, observed.saved_revision);
    assert_eq!(document.buffer, unsaved);
    let value = serde_json::to_value(observed).unwrap();
    assert!(value.get("buffer").is_none());
    assert!(value.get("source_sha256").is_none());
    assert!(value.get("frame_generation").is_none());
}

#[test]
fn loaded_html_observation_preserves_reference_and_unknown_disk_hash() {
    let mut document = Document::new("reference.html", "<p>reference</p>");
    document.is_reference = true;
    document.last_imported_disk_hash = None;
    let observed = HtmlLoadedSourceObservation::capture(Some(&document), u64::MAX).unwrap();
    assert!(observed.is_reference);
    assert_eq!(observed.preview_input_fnv1a64, "ffffffffffffffff");
    assert!(observed.last_imported_disk_fnv1a64.is_none());
    assert!(!observed.dirty);
}

#[test]
fn document_observation_reports_actual_identity_and_absent_frame_without_loading_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.pdf");
    std::fs::write(
        &path,
        include_bytes!(
            "../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        ),
    )
    .unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let expected_uri = source.uri.clone();
    let expected_revision = source.revision.clone();
    let mut pane = PreviewPane::default();
    assert!(DocumentReceiptObservation::capture(&pane).is_none());
    pane.full_render_document_source(source, false);
    let observed = DocumentReceiptObservation::capture(&pane).unwrap();
    assert_eq!(observed.uri, expected_uri);
    assert_eq!(observed.source_revision, expected_revision);
    assert_eq!(
        pane.document_resource_identity(),
        Some((observed.uri.clone(), observed.generation))
    );
    assert!(observed.frame.is_none());
    assert!(!observed.idle);
}

#[test]
fn normal_snapshot_keeps_additional_receipt_observations_empty() {
    let mut app = super::super::KatanaApp::new(crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        Default::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    ));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("snapshot.pdf");
    std::fs::write(
        &path,
        include_bytes!(
            "../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        ),
    )
    .unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_source(DocumentSurfaceSource::local(&path).unwrap(), false);
    app.tab_previews.push(super::super::TabPreviewCache {
        path,
        pane,
        hash: 0,
    });
    let (snapshot, _) = UiResourceSnapshot::capture(&app);
    assert_eq!(snapshot.host.document_surfaces, 1);
    assert_eq!(snapshot.document_surfaces.len(), 1);
    assert!(snapshot.document_observations.is_empty());
    assert!(snapshot.html_observations.is_empty());
    let (receipt, _) = UiResourceSnapshot::capture_for_receipt(&app).unwrap();
    assert_eq!(receipt.document_surfaces, snapshot.document_surfaces);
    assert_eq!(receipt.document_observations.len(), 1);
    assert!(receipt.document_observations[0].frame.is_none());
    assert!(!receipt.document_observations[0].idle);
    let serialized = serde_json::to_value(receipt).unwrap();
    assert!(serialized["document_observations"][0]["frame"].is_null());
    assert!(
        serialized["document_observations"][0]
            .get("frame_generation")
            .is_none()
    );
}
