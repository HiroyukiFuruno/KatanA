use super::fixtures::{
    INITIAL_HARNESS_FRAME_STEPS, await_html_frame, await_office_frame, make_harness,
};
use crate::app::DocumentOps;
use crate::preview_pane::DocumentWorkerLifecycle;
use egui_kittest::Harness;
use std::path::Path;
use std::time::{Duration, Instant};

#[path = "png_explorer/assertions.rs"]
mod assertions;
#[path = "png_explorer/fixture.rs"]
mod fixture;
#[path = "png_explorer/inflight_close_tests.rs"]
mod inflight_close_tests;
#[path = "png_explorer/input.rs"]
mod input;
#[path = "png_explorer/mixed_directory_tests.rs"]
mod mixed_directory_tests;
#[path = "png_explorer/on_demand.rs"]
mod on_demand;
#[path = "png_explorer/png_directory_tests.rs"]
mod png_directory_tests;
use fixture::ExplorerImageOps;

const LOAD_TIMEOUT: Duration = Duration::from_secs(15);
const OFFICE_NAME: &str = "representative.pptx";
const IMAGE_NAME: &str = "explorer-sample.png";

#[test]
fn explorer_click_paints_generated_png_during_office_and_html_loading() {
    let _render_lock = crate::test_render_env::RenderEnvLock::lock();
    let office = input::LoadingFixtureInputs::office_path();
    assert!(
        office.is_file(),
        "public representative PPTX fixture is missing"
    );
    run_office_case(&office);
    run_html_case();
}

fn run_office_case(office: &Path) {
    let temporary = tempfile::tempdir().expect("isolated Office harness configuration");
    let mut harness = make_harness(&temporary);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    let image_path =
        ExplorerImageOps::prepare_workspace(&mut harness, &temporary.path().join("workspace"));
    let office_path = temporary.path().join(OFFICE_NAME);
    std::fs::copy(office, &office_path).expect("copy public representative PPTX");
    let baseline = DocumentWorkerLifecycle::live_count();
    assert_eq!(
        baseline, 0,
        "isolated public-fixture harness starts without document workers"
    );
    harness
        .state_mut()
        .handle_select_document(office_path.clone(), true);
    await_office_inflight(&mut harness, &office_path, baseline);
    let selected =
        ExplorerImageOps::click_png_row(&mut harness, &image_path, Some(&office_path), baseline);
    ExplorerImageOps::assert_texture_painted(&mut harness, &image_path, selected);
    /* WHY: PNG到達と、表示完了後の既存close契約を別の段階として検証する。 */
    harness
        .state_mut()
        .handle_select_document(office_path, true);
    await_office_frame(&mut harness, "pptx");
    ExplorerImageOps::close_all(&mut harness, baseline, "PPTX");
}

fn await_office_inflight(
    harness: &mut Harness<'_, crate::shell::KatanaApp>,
    path: &Path,
    baseline: usize,
) {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    loop {
        ExplorerImageOps::step(harness);
        let office = harness
            .state()
            .tab_previews
            .iter()
            .find(|tab| tab.path == path);
        let pane = office.expect("Office preview after document selection");
        assert!(
            pane.pane.document_failure_for_test().is_none(),
            "PPTX loading failed"
        );
        if pane.pane.document_surface.is_some()
            && pane.pane.document_frame_state_for_test().is_none()
            && DocumentWorkerLifecycle::live_count() > baseline
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "PPTX worker did not become live without a frame within 15 seconds"
        );
        std::thread::yield_now();
    }
}

fn run_html_case() {
    let temporary = tempfile::tempdir().expect("isolated HTML harness configuration");
    let mut harness = make_harness(&temporary);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    let image_path =
        ExplorerImageOps::prepare_workspace(&mut harness, &temporary.path().join("workspace"));
    let html_path = temporary.path().join("loading.html");
    input::LoadingFixtureInputs::create_html(&html_path);
    let baseline = DocumentWorkerLifecycle::live_count();
    assert_eq!(
        baseline, 0,
        "isolated HTML harness starts without document workers"
    );
    harness
        .state_mut()
        .handle_select_document(html_path.clone(), true);
    await_html_inflight(&mut harness);
    let selected = ExplorerImageOps::click_png_row(&mut harness, &image_path, None, 0);
    let html = &harness
        .state()
        .tab_previews
        .iter()
        .find(|tab| tab.path == html_path)
        .expect("retained HTML")
        .pane;
    assert!(html.html_browser_frame_generation().is_none());
    assert_eq!(html.html_browser_is_idle(), Some(false));
    ExplorerImageOps::assert_texture_painted(&mut harness, &image_path, selected);
    harness.state_mut().handle_select_document(html_path, true);
    await_html_frame(&mut harness);
    ExplorerImageOps::close_all(&mut harness, baseline, "HTML");
}

fn await_html_inflight(harness: &mut Harness<'_, crate::shell::KatanaApp>) {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    loop {
        let in_flight = harness
            .state()
            .html_browser_frame_generation_for_test()
            .is_none()
            && harness.state().html_browser_is_idle_for_test() == Some(false);
        if in_flight {
            return;
        }
        assert!(
            harness
                .state()
                .html_browser_frame_generation_for_test()
                .is_none(),
            "HTML finished before an in-flight frame could be observed"
        );
        assert!(
            Instant::now() < deadline,
            "HTML browser did not enter pending state within 15 seconds"
        );
        ExplorerImageOps::step(harness);
        std::thread::yield_now();
    }
}
