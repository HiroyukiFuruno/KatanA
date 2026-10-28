#![cfg(unix)]

use super::*;
use katana_core::system::ProcessService;
use std::sync::mpsc;
use std::time::{Duration, Instant};

mod support;
use support::{
    assert_global_queue_full, make_png, open_fifo_writer, paint_target, wait_for_scheduled_repaint,
    wait_until_watched, write_png_payload,
};

const CHILD_ENV: &str = "KATANA_IMAGE_QUEUE_LIFECYCLE_CHILD";
const SCENARIO_ENV: &str = "KATANA_IMAGE_QUEUE_LIFECYCLE_SCENARIO";
const TEST_TIMEOUT: Duration = Duration::from_secs(5);
const QUEUED_FIRST_COLOR: [u8; 4] = [40, 90, 130, u8::MAX];
const QUEUED_SECOND_COLOR: [u8; 4] = [60, 110, 150, u8::MAX];
const TARGET_COLOR: [u8; 4] = [80, 130, 170, u8::MAX];
const QUEUED_FIRST_SIZE: [u32; 2] = [2, 2];
const QUEUED_SECOND_SIZE: [u32; 2] = [3, 3];
const TARGET_SIZE: [u32; 2] = [4, 4];
const TARGET_TEXTURE_SIZE: [usize; 2] = [4, 4];

#[test]
fn full_global_queue_retries_visible_image_after_worker_completion() {
    run_child("completion");
}

#[test]
fn full_global_queue_retries_visible_image_after_queued_work_is_cancelled() {
    run_child("cancellation");
}

fn run_child(scenario: &str) {
    let child_test = format!(
        "{}::global_queue_lifecycle_child",
        module_path!()
            .strip_prefix("katana_ui::")
            .expect("module path")
    );
    let executable = std::env::current_exe().expect("test executable");
    let output = ProcessService::create_command(executable.to_str().expect("executable path"))
        .args(["--exact", &child_test, "--nocapture"])
        .env(CHILD_ENV, "1")
        .env(SCENARIO_ENV, scenario)
        .output()
        .expect("spawn isolated global-pool child test");
    assert!(String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn global_queue_lifecycle_child() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let _render_guard = crate::test_render_env::RenderEnvLock::lock();
    let scenario = std::env::var(SCENARIO_ENV).expect("child scenario");
    QueueLifecycleCase::new().recover(&scenario);
}

struct QueueLifecycleCase {
    directory: tempfile::TempDir,
    paths: QueuePaths,
    context: egui::Context,
    repaint_rx: mpsc::Receiver<(Instant, Duration)>,
    blocker: LocalImageLoader,
    fifo_writer: Option<std::fs::File>,
    queued: LocalImageLoader,
    target: LocalImageLoader,
    viewer: crate::preview_pane::ViewerState,
}

struct QueuePaths {
    fifo: std::path::PathBuf,
    queued: [std::path::PathBuf; REQUEST_CAPACITY],
    target: std::path::PathBuf,
}

impl QueueLifecycleCase {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("isolated fixture directory");
        let paths = write_queue_fixtures(directory.path());
        let (context, repaint_rx) = repaint_context();
        let (blocker, fifo_writer) = start_fifo_blocker(&context, &paths.fifo);
        let queued = fill_production_queue(&context, &paths.queued, &paths.target);
        let (target, viewer) = start_visible_target(&context, &paths.target);
        Self {
            directory,
            paths,
            context,
            repaint_rx,
            blocker,
            fifo_writer: Some(fifo_writer),
            queued,
            target,
            viewer,
        }
    }

    fn recover(mut self, scenario: &str) {
        if scenario == "cancellation" {
            self.blocker.reset();
            self.queued.reset();
        } else {
            assert_eq!(scenario, "completion");
        }
        let mut fifo_writer = self.fifo_writer.take().expect("FIFO writer still open");
        write_png_payload(&mut fifo_writer, self.directory.path());
        drop(fifo_writer);
        self.wait_until_target_ready();
        assert_eq!(
            self.viewer.texture.as_ref().expect("target texture").size(),
            TARGET_TEXTURE_SIZE
        );
    }

    fn wait_until_target_ready(&mut self) {
        let deadline = Instant::now() + TEST_TIMEOUT;
        while self.viewer.texture.is_none() {
            wait_for_scheduled_repaint(&self.repaint_rx, deadline);
            let _ = paint_target(
                &self.target,
                &self.context,
                &self.paths.target,
                &mut self.viewer,
            );
            assert!(
                Instant::now() < deadline,
                "visible target did not recover after production queue capacity returned"
            );
        }
    }
}

fn write_queue_fixtures(directory: &Path) -> QueuePaths {
    let fifo = directory.join("worker-blocker.png");
    let queued = [
        directory.join("queued-first.png"),
        directory.join("queued-second.png"),
    ];
    let target = directory.join("visible-target.png");
    make_png(&queued[0], QUEUED_FIRST_COLOR, QUEUED_FIRST_SIZE);
    make_png(&queued[1], QUEUED_SECOND_COLOR, QUEUED_SECOND_SIZE);
    make_png(&target, TARGET_COLOR, TARGET_SIZE);
    assert!(
        ProcessService::create_command("mkfifo")
            .arg(&fifo)
            .status()
            .expect("create FIFO")
            .success(),
        "create FIFO blocker"
    );
    QueuePaths {
        fifo,
        queued,
        target,
    }
}

fn repaint_context() -> (egui::Context, mpsc::Receiver<(Instant, Duration)>) {
    let context = egui::Context::default();
    let (repaint_tx, repaint_rx) = mpsc::channel();
    context.set_request_repaint_callback(move |info| {
        let _ = repaint_tx.send((Instant::now(), info.delay));
    });
    (context, repaint_rx)
}

fn start_fifo_blocker(context: &egui::Context, fifo: &Path) -> (LocalImageLoader, std::fs::File) {
    let blocker = LocalImageLoader::default();
    assert!(matches!(
        blocker.texture(context, fifo, crate::theme_bridge::WHITE, 0),
        LocalTextureStatus::Pending
    ));
    let writer = open_fifo_writer(&blocker, context, fifo);
    (blocker, writer)
}

fn fill_production_queue(
    context: &egui::Context,
    paths: &[std::path::PathBuf; REQUEST_CAPACITY],
    probe_path: &Path,
) -> LocalImageLoader {
    let queued = LocalImageLoader::default();
    for path in paths {
        wait_until_watched(&queued, path);
        assert!(matches!(
            queued.texture(context, path, crate::theme_bridge::WHITE, 0),
            LocalTextureStatus::Pending
        ));
        assert!(queued.is_pending(&RequestKey {
            path: path.clone(),
            background: crate::theme_bridge::WHITE,
        }));
    }
    assert_global_queue_full(&queued, probe_path);
    queued
}

fn start_visible_target(
    context: &egui::Context,
    path: &Path,
) -> (LocalImageLoader, crate::preview_pane::ViewerState) {
    let target = LocalImageLoader::default();
    wait_until_watched(&target, path);
    assert!(matches!(
        target.texture(context, path, crate::theme_bridge::WHITE, 0),
        LocalTextureStatus::Pending
    ));
    let mut viewer = crate::preview_pane::ViewerState::default();
    assert!(paint_target(&target, context, path, &mut viewer).is_none());
    assert!(
        viewer.texture.is_none(),
        "target should visibly remain pending"
    );
    (target, viewer)
}
