use super::super::fixtures::{INITIAL_HARNESS_FRAME_STEPS, make_harness};
use super::LOAD_TIMEOUT;
use super::fixture::ExplorerImageOps;
use egui_kittest::{Harness, kittest::Queryable};
use std::path::Path;
use std::time::{Duration, Instant};

#[path = "repaint_driver.rs"]
mod repaint_driver;
use repaint_driver::RepaintDriver;

pub(super) fn measure(cohort: &str, root: &Path, image: &Path) -> Duration {
    let configuration = tempfile::tempdir().expect("isolated on-demand cohort settings");
    let mut harness = make_harness(&configuration);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    ExplorerImageOps::open_workspace(&mut harness, root);
    ExplorerImageOps::step(&mut harness);
    eprintln!(
        "png_explorer_on_demand_cohort cohort={cohort} root={} image={}",
        root.display(),
        image.display()
    );
    let elapsed = click_and_await_mesh(&mut harness, image);
    eprintln!(
        "png_explorer_on_demand_result cohort={cohort} first_mesh_ms={}",
        elapsed.as_secs_f64() * 1000.0
    );
    ExplorerImageOps::close_all(&mut harness, 0, "on-demand PNG directory comparison");
    elapsed
}

pub(super) fn click_and_await_mesh(
    harness: &mut Harness<'_, crate::shell::KatanaApp>,
    image: &Path,
) -> Duration {
    let mut driver = RepaintDriver::install(&harness.ctx);
    let name = image
        .file_name()
        .and_then(|name| name.to_str())
        .expect("PNG filename is UTF-8");
    let started = Instant::now();
    let initial_pass = harness.ctx.cumulative_pass_nr();
    harness.get_by_label(&format!("file {name}")).click();
    /* WHY: 初回だけ実入力を処理し、その後はcallbackまたはviewportの期限到達時に限って進める。 */
    ExplorerImageOps::step(harness);
    let elapsed = await_mesh(harness, &mut driver, image, started, initial_pass);
    eprintln!(
        "png_explorer_on_demand_loaded path={} first_mesh_ms={} callbacks={}",
        image.display(),
        elapsed.as_secs_f64() * 1000.0,
        driver.callbacks
    );
    elapsed
}

pub(super) fn initialize_preview(harness: &mut Harness<'_, crate::shell::KatanaApp>, path: &Path) {
    let mut driver = RepaintDriver::install(&harness.ctx);
    let started = Instant::now();
    let deadline = started + LOAD_TIMEOUT;
    ExplorerImageOps::step(harness);
    loop {
        assert!(
            Instant::now() < deadline,
            "preview initialization exceeded 15 seconds: {}",
            path.display()
        );
        let pane = &harness
            .state()
            .tab_previews
            .iter()
            .find(|preview| preview.path == path)
            .expect("opened preview")
            .pane;
        assert!(
            pane.document_failure_for_test().is_none(),
            "preview initialization failed: {} {:?}",
            path.display(),
            pane.document_failure_for_test()
        );
        if pane.document_surface.is_some() || pane.html_browser.is_some() {
            eprintln!(
                "png_explorer_mixed_initialized path={} elapsed_ms={} document_frame={:?} html_frame={:?}",
                path.display(),
                started.elapsed().as_secs_f64() * 1000.0,
                pane.document_frame_state_for_test(),
                pane.html_browser_frame_generation()
            );
            return;
        }
        driver.record_output(harness.output(), started);
        driver.wait_until_due(&harness.ctx, started, deadline);
        ExplorerImageOps::step(harness);
    }
}

fn await_mesh(
    harness: &mut Harness<'_, crate::shell::KatanaApp>,
    driver: &mut RepaintDriver,
    image: &Path,
    started: Instant,
    initial_pass: u64,
) -> Duration {
    let deadline = started + LOAD_TIMEOUT;
    let mut selected = false;
    loop {
        let frames = harness
            .ctx
            .cumulative_pass_nr()
            .saturating_sub(initial_pass);
        assert!(
            Instant::now() < deadline,
            "on-demand PNG did not reach a mesh within 15 seconds"
        );
        if !selected && harness.state().state.active_path().as_deref() == Some(image) {
            selected = true;
            eprintln!(
                "png_explorer_on_demand_selection elapsed_ms={} frames={frames}",
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        if ExplorerImageOps::painted_texture(harness, image).is_some() {
            assert!(selected, "painted PNG must be the selected document");
            eprintln!("png_explorer_on_demand_mesh frames={frames}");
            return started.elapsed();
        }
        driver.record_output(harness.output(), started);
        driver.wait_until_due(&harness.ctx, started, deadline);
        ExplorerImageOps::step(harness);
    }
}
