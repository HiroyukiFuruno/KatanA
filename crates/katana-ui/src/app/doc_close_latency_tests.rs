mod fixtures;

use crate::app::DocumentOps;
use crate::preview_pane::DocumentWorkerLifecycle;
use fixtures::{
    CLOSE_TIMEOUT, Fixture, INITIAL_HARNESS_FRAME_STEPS, await_html_frame, await_office_frame,
    await_workers_idle, fixtures, make_harness,
};
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn actual_html_and_office_close_paths_record_frame_drop_and_worker_idle_timings() {
    let _render_lock = crate::test_render_env::RenderEnvLock::lock();
    let worker = std::env::var_os("KATANA_KDV_OFFICE_WORKER")
        .map(PathBuf::from)
        .expect("real Office worker path must be supplied by with-office-test-worker.sh");
    assert!(
        worker.is_file(),
        "real Office worker is missing: {}",
        worker.display()
    );

    let temporary = tempfile::tempdir().expect("create isolated KatanA close-timing fixture root");
    let fixtures: Vec<Fixture> = fixtures(temporary.path());
    let mut harness = make_harness(&temporary);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        harness.step();
    }

    let baseline_workers = DocumentWorkerLifecycle::live_count();
    assert_eq!(
        baseline_workers, 0,
        "the isolated harness must start without document workers"
    );
    for fixture in fixtures {
        assert!(
            fixture.path.is_file(),
            "fixture copy missing: {}",
            fixture.path.display()
        );
        harness
            .state_mut()
            .handle_select_document(fixture.path.clone(), true);
        let loaded = if fixture.kind == "html" {
            format!(
                "format=html browser_frame_generation={}",
                await_html_frame(&mut harness)
            )
        } else {
            await_office_frame(&mut harness, fixture.kind)
        };
        let before_resources = harness.state_mut().preview_resource_counts_for_test();
        assert_eq!(
            before_resources.0, 1,
            "one real preview should be open for {}",
            fixture.kind
        );
        assert!(
            before_resources.3 > 0,
            "{} must have a rendered frame",
            fixture.kind
        );
        if fixture.kind == "html" {
            assert!(
                before_resources.1 > 0,
                "HTML browser surface was not loaded"
            );
        } else {
            assert!(
                before_resources.2 > 0,
                "Office document surface was not loaded"
            );
            assert!(
                DocumentWorkerLifecycle::live_count() > baseline_workers,
                "{} was not loaded by a real document worker",
                fixture.kind
            );
        }
        let (workers_before, kdv_before) =
            harness.state_mut().document_lifecycle_resources_for_test();
        let index = harness
            .state_mut()
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == fixture.path)
            .expect("fixture is an open KatanA document");
        let close_started = Instant::now();
        let close_deadline = close_started + CLOSE_TIMEOUT;
        harness.state_mut().force_close_document(index);
        let close_ms = close_started.elapsed().as_millis();
        fixtures::assert_before_deadline(close_deadline, fixture.kind, "force close");
        harness.step();
        fixtures::assert_before_deadline(close_deadline, fixture.kind, "close and next frame");
        let close_to_next_frame_ms = close_started.elapsed().as_millis();
        let after_resources = harness.state_mut().preview_resource_counts_for_test();
        assert!(
            !harness
                .state_mut()
                .state
                .document
                .open_documents
                .iter()
                .any(|document| document.path == fixture.path),
            "{} remained open after KatanA force close",
            fixture.kind
        );
        assert_eq!(
            after_resources,
            (0, 0, 0, 0, 0, 0),
            "{} left a preview resource after close",
            fixture.kind
        );
        let idle_wait_ms =
            await_workers_idle(&mut harness, baseline_workers, fixture.kind, close_deadline);
        let (workers_after, kdv_after) =
            harness.state_mut().document_lifecycle_resources_for_test();
        assert_eq!(
            workers_after, baseline_workers,
            "{} worker did not become idle",
            fixture.kind
        );
        eprintln!(
            "ISSUE386_REAL_CLOSE format={} path={} loaded_frame={} resources_before={:?} resources_after={:?} workers_before={} workers_after={} kdv_before={:?} kdv_after={:?} force_close_ms={} close_to_next_frame_ms={} worker_idle_wait_ms={}",
            fixture.kind,
            fixture.path.display(),
            loaded,
            before_resources,
            after_resources,
            workers_before,
            workers_after,
            kdv_before,
            kdv_after,
            close_ms,
            close_to_next_frame_ms,
            idle_wait_ms
        );
    }
}

#[cfg(test)]
mod deadline_tests {
    use super::fixtures::{INITIAL_HARNESS_FRAME_STEPS, await_workers_idle, make_harness};
    use crate::preview_pane::DocumentWorkerLifecycle;
    use std::panic::AssertUnwindSafe;
    use std::time::{Duration, Instant};

    #[test]
    fn expired_deadline_is_rejected_even_when_worker_is_already_idle() {
        let _render_lock = crate::test_render_env::RenderEnvLock::lock();
        let temporary = tempfile::tempdir().expect("create isolated close-deadline fixture root");
        let mut harness = make_harness(&temporary);
        for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
            harness.step();
        }

        let baseline_workers = DocumentWorkerLifecycle::live_count();
        let lifecycle_resources = harness.state_mut().document_lifecycle_resources_for_test();
        assert_eq!(
            lifecycle_resources.0, baseline_workers,
            "the isolated harness must have no worker beyond its baseline"
        );
        assert_eq!(
            harness.state_mut().preview_resource_counts_for_test(),
            (0, 0, 0, 0, 0, 0),
            "the isolated harness must have no open preview"
        );

        let expired_deadline = Instant::now() - Duration::from_secs(1);
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            await_workers_idle(&mut harness, baseline_workers, "HTML", expired_deadline);
        }));

        assert!(
            result.is_err(),
            "an expired close deadline must fail even if workers are already idle"
        );
    }
}
