use super::ResourceReceiptState;
use crate::{
    app_state::AppState,
    shell::{
        KatanaApp,
        resource_snapshot::{
            DocumentSurfaceIdentity, HtmlSurfaceReceiptObservation, UiResourceSnapshot,
            completed_frame_capture_count_for_test, reset_completed_frame_capture_count_for_test,
        },
    },
};
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

fn snapshot() -> UiResourceSnapshot {
    UiResourceSnapshot {
        document_surfaces: vec![DocumentSurfaceIdentity {
            uri: "file:///fixture/example.xlsx".to_owned(),
            generation: 7,
        }],
        ..UiResourceSnapshot::default()
    }
}

fn make_app() -> KatanaApp {
    KatanaApp::new(AppState::new(
        Default::default(),
        Default::default(),
        Default::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    ))
}

#[test]
fn disabled_completed_frame_callback_does_not_capture() {
    let mut app = make_app();
    app.resource_receipt = None;
    reset_completed_frame_capture_count_for_test();
    app.record_completed_frame_resources();
    assert_eq!(completed_frame_capture_count_for_test(), 0);
}

#[test]
fn capture_counter_is_scoped_to_its_calling_thread() {
    reset_completed_frame_capture_count_for_test();
    std::thread::spawn(|| {
        let app = make_app();
        let _ = UiResourceSnapshot::capture(&app);
        assert_eq!(completed_frame_capture_count_for_test(), 1);
    })
    .join()
    .unwrap();
    assert_eq!(completed_frame_capture_count_for_test(), 0);
}

#[test]
fn disabled_configuration_never_creates_a_writer() {
    assert!(
        ResourceReceiptState::configuration(false, None, None)
            .unwrap()
            .is_none()
    );
    assert!(
        ResourceReceiptState::configuration(
            false,
            Some(PathBuf::from("/existing/run-owned-dir")),
            Some(OsString::from("run-1")),
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn receipt_uses_real_file_and_preserves_identity() {
    let directory = tempfile::tempdir().unwrap();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(directory.path().to_path_buf()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();
    receipt.record(snapshot(), 12);
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("latest.json")).unwrap())
            .unwrap();
    assert_eq!(value["run_id"], "run-1");
    assert_eq!(value["seq"], 1);
    assert_eq!(
        value["document_surfaces"][0]["uri"],
        "file:///fixture/example.xlsx"
    );
    assert_eq!(value["document_surfaces"][0]["generation"], 7);
}

#[test]
fn receipt_preserves_html_adapter_identity_and_raw_geometry() {
    let directory = tempfile::tempdir().unwrap();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(directory.path().to_path_buf()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();
    let mut observed = snapshot();
    observed
        .html_observations
        .push(HtmlSurfaceReceiptObservation {
            source_uri: "file:///fixture/example.html".to_owned(),
            katana_adapter_instance_id: 9,
            katana_renderer_session_epoch: 3,
            frame_generation: 12,
            viewport_width: 900.0,
            viewport_height: 700.0,
            scroll_y: 48.5,
            content_height: 2_048.0,
            idle: true,
            loaded_source: None,
        });

    receipt.record(observed, 12);

    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("latest.json")).unwrap())
            .unwrap();
    let observation = &value["html_observations"][0];
    assert_eq!(observation["source_uri"], "file:///fixture/example.html");
    assert_eq!(observation["katana_adapter_instance_id"], 9);
    assert_eq!(observation["katana_renderer_session_epoch"], 3);
    assert_eq!(observation["frame_generation"], 12);
    assert_eq!(observation["viewport_width"], 900.0);
    assert_eq!(observation["viewport_height"], 700.0);
    assert_eq!(observation["scroll_y"], 48.5);
    assert_eq!(observation["content_height"], 2_048.0);
    assert_eq!(observation["idle"], true);
}

#[test]
fn capture_error_rejects_the_diagnostic_run_without_publishing_zeroes() {
    let directory = tempfile::tempdir().unwrap();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(directory.path().to_path_buf()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();

    receipt.record_capture_error("non-finite HTML geometry".to_owned());

    assert!(receipt.failed);
    assert_eq!(receipt.sequence, 0);
    assert!(receipt.last_snapshot.is_none());
    assert!(!directory.path().join("latest.json").exists());
}

#[test]
fn unchanged_or_early_snapshot_does_not_publish() {
    let directory = tempfile::tempdir().unwrap();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(directory.path().to_path_buf()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();
    let first = snapshot();
    assert!(receipt.begin_capture());
    receipt.record(first.clone(), 1);
    receipt.record(first, 1);
    assert_eq!(receipt.sequence, 1);
    let mut changed = snapshot();
    changed.document_surfaces[0].generation = 8;
    assert!(!receipt.begin_capture());
    assert_eq!(receipt.sequence, 1);
    receipt.last_capture_at = Some(Instant::now() - Duration::from_millis(100));
    assert!(receipt.begin_capture());
    receipt.record(changed, 1);
    assert_eq!(receipt.sequence, 2);
    let entries = std::fs::read_dir(directory.path()).unwrap().count();
    assert_eq!(
        entries, 1,
        "atomic replace must not retain a temporary receipt"
    );
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("latest.json")).unwrap())
            .unwrap();
    assert_eq!(value["seq"], 2);
}

#[test]
fn failed_publish_is_not_marked_as_sent() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().to_path_buf();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(path.clone()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();
    std::fs::remove_dir(&path).unwrap();
    receipt.record(snapshot(), 1);
    assert_eq!(receipt.sequence, 0);
    assert!(receipt.last_snapshot.is_none());
    assert!(receipt.failed);
    receipt.record(snapshot(), 1);
    assert_eq!(receipt.sequence, 0, "failed diagnostic runs stay rejected");
}

#[cfg(unix)]
#[test]
fn receipt_file_is_private_on_posix() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let mut receipt = ResourceReceiptState::configuration(
        true,
        Some(directory.path().to_path_buf()),
        Some(OsString::from("run-1")),
    )
    .unwrap()
    .unwrap();
    receipt.record(snapshot(), 1);
    let permissions = std::fs::metadata(directory.path().join("latest.json"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(permissions & 0o777, 0o600);
}

#[test]
fn invalid_directory_or_run_id_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("not-a-directory");
    std::fs::write(&file, b"x").unwrap();
    assert!(
        ResourceReceiptState::configuration(true, Some(file), Some(OsString::from("run-1")),)
            .is_err()
    );
    for run_id in ["not allowed".to_owned(), "a".repeat(65)] {
        assert!(
            ResourceReceiptState::configuration(
                true,
                Some(directory.path().to_path_buf()),
                Some(OsString::from(run_id)),
            )
            .is_err()
        );
    }
}
