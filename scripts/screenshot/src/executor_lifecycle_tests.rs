use super::{ObservedFrameProgress, RuntimeSnapshot, assert_runtime_snapshot};
use crate::request::AssertRuntimeSnapshotStep;
use katana_ui::shell::DocumentResourceSnapshotForTest;

fn idle_snapshot() -> RuntimeSnapshot {
    RuntimeSnapshot {
        rss_kib: 0,
        ui_frame: 0,
        observed: ObservedFrameProgress::default(),
        previews: 0,
        html_surfaces: 0,
        document_surfaces: 0,
        office_workers: 0,
        document_workers: 0,
        kdv_resources: DocumentResourceSnapshotForTest::default(),
        frames: 0,
        textures: 0,
        cache_entries: 0,
    }
}

fn idle_limits() -> AssertRuntimeSnapshotStep {
    AssertRuntimeSnapshotStep {
        baseline: "idle".to_owned(),
        max_rss_delta_kib: 0,
        min_ui_frame_delta: 0,
        min_html_frames_observed: 0,
        min_document_frames_observed: 0,
        max_previews: 0,
        max_html_surfaces: 0,
        max_document_surfaces: 0,
        max_office_workers: 0,
        max_frames: 0,
        max_textures: 0,
        max_cache_entries: 0,
    }
}

#[test]
fn idle_acceptance_rejects_a_worker_after_tab_removal() {
    let idle = idle_snapshot();
    let current = RuntimeSnapshot {
        document_workers: 1,
        ..idle
    };
    let error = assert_runtime_snapshot(&idle, &current, &idle_limits()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("document lifecycle resources remain")
    );
    assert!(assert_runtime_snapshot(&idle, &idle, &idle_limits()).is_ok());
}

#[test]
fn idle_acceptance_rejects_every_published_kdv_resource() {
    let updates: [fn(&mut DocumentResourceSnapshotForTest); 8] = [
        |r| r.live_document_sessions = 1,
        |r| r.live_spreadsheet_workers = 1,
        |r| r.live_worker_workspaces = 1,
        |r| r.retained_artifact_bytes = 1,
        |r| r.cached_page_count = 1,
        |r| r.cached_page_bytes = 1,
        |r| r.cached_spreadsheet_cell_count = 1,
        |r| r.cached_spreadsheet_cell_bytes = 1,
    ];
    let idle = idle_snapshot();
    for update in updates {
        let mut current = idle;
        update(&mut current.kdv_resources);
        let error = assert_runtime_snapshot(&idle, &current, &idle_limits()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("document lifecycle resources remain")
        );
    }
}
