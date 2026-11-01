use super::{
    ExplorerImageOps, INITIAL_HARNESS_FRAME_STEPS, await_html_frame, await_html_inflight,
    await_office_frame, await_office_inflight, input, make_harness,
};
use crate::app::DocumentOps;
use crate::preview_pane::DocumentWorkerLifecycle;
use crate::shell::KatanaApp;
use egui_kittest::Harness;
use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn real_office_inflight_close_preserves_other_tab_and_reopens() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    run_case(true);
}

#[test]
fn real_html_inflight_close_preserves_other_tab_and_reopens() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    run_case(false);
}

fn run_case(office: bool) {
    let temporary = tempfile::tempdir().expect("isolated in-flight close fixture");
    let mut harness = make_harness(&temporary);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    let (target, other) = prepare_files(temporary.path(), office);
    exercise_close_and_reopen(&mut harness, &target, &other, office);
}

fn prepare_files(root: &Path, office: bool) -> (std::path::PathBuf, std::path::PathBuf) {
    let other = root.join("retained.md");
    std::fs::write(&other, "# retained tab\n").expect("write isolated other tab");
    let target = root.join(if office {
        "inflight.pptx"
    } else {
        "inflight.html"
    });
    if office {
        std::fs::copy(input::LoadingFixtureInputs::office_path(), &target)
            .expect("copy PPTX fixture");
    } else {
        input::LoadingFixtureInputs::create_html(&target);
    }
    (target, other)
}

fn exercise_close_and_reopen(
    harness: &mut Harness<'_, KatanaApp>,
    target: &Path,
    other: &Path,
    office: bool,
) {
    harness
        .state_mut()
        .handle_select_document(other.to_path_buf(), true);
    let baseline = DocumentWorkerLifecycle::live_count();
    harness
        .state_mut()
        .handle_select_document(target.to_path_buf(), true);
    if office {
        await_office_inflight(harness, target, baseline);
    } else {
        await_html_inflight(harness);
    }
    close_inflight(harness, target, other);
    harness
        .state_mut()
        .handle_select_document(target.to_path_buf(), true);
    if office {
        await_office_frame(harness, "pptx");
    } else {
        await_html_frame(harness);
    }
    assert_other_retained(harness, other);
    assert_eq!(harness.state().state.active_path().as_deref(), Some(target));
}

fn assert_other_retained(harness: &Harness<'_, KatanaApp>, other: &Path) {
    assert!(
        harness
            .state()
            .state
            .document
            .open_documents
            .iter()
            .any(|doc| doc.path == other)
    );
}

fn close_inflight(harness: &mut Harness<'_, KatanaApp>, target: &Path, other: &Path) {
    let index = harness
        .state()
        .state
        .document
        .open_documents
        .iter()
        .position(|doc| doc.path == target)
        .expect("loading document is open");
    let frame = harness.ctx.cumulative_frame_nr();
    let started = Instant::now();
    harness.state_mut().force_close_document(index);
    harness.step();
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "close and next UI frame exceeded deadline"
    );
    assert!(
        harness.ctx.cumulative_frame_nr() > frame,
        "UI did not advance"
    );
    assert_closed(harness, target);
    assert_other_retained(harness, other);
    assert_eq!(harness.state().state.active_path().as_deref(), Some(other));
}

fn assert_closed(harness: &Harness<'_, KatanaApp>, target: &Path) {
    assert!(
        !harness
            .state()
            .state
            .document
            .open_documents
            .iter()
            .any(|doc| doc.path == target)
    );
    assert!(
        !harness
            .state()
            .tab_previews
            .iter()
            .any(|preview| preview.path == target)
    );
}
