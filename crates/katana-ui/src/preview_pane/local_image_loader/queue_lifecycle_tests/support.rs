use super::{GlobalPool, LocalImageLoader, RequestKey, TEST_TIMEOUT, Work, egui, mpsc};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

const RGBA_CHANNEL_COUNT: usize = 4;
const TEST_IMAGE_ID: usize = 0;
const TEST_MAX_TEXTURE_SIDE: usize = 8192;
const TEST_VIEWPORT_WIDTH: f32 = 640.0;
const TEST_VIEWPORT_HEIGHT: f32 = 480.0;
const FIFO_PAYLOAD_COLOR: [u8; 4] = [u8::MAX, 20, 30, u8::MAX];
const FIFO_PAYLOAD_SIZE: [u32; 2] = [2, 2];

pub(super) fn wait_until_watched(loader: &LocalImageLoader, path: &Path) {
    let deadline = Instant::now() + TEST_TIMEOUT;
    loop {
        loader.poll(0);
        if loader
            .inner
            .watched_paths
            .lock()
            .expect("watched paths lock")
            .contains(path)
        {
            return;
        }
        let _ = loader.ensure_watched(path);
        assert!(
            Instant::now() < deadline,
            "watcher did not register {path:?}"
        );
        std::thread::yield_now();
    }
}

pub(super) fn assert_global_queue_full(loader: &LocalImageLoader, probe_path: &Path) {
    let work = Work {
        owner: Arc::downgrade(&loader.inner),
        result_tx: loader.inner.result_tx.clone(),
        key: RequestKey {
            path: probe_path.to_path_buf(),
            background: crate::theme_bridge::WHITE,
        },
        generation: loader.generation(),
        revision: loader.path_revision(probe_path),
    };
    let pool = GlobalPool::get().expect("production image pool");
    assert!(matches!(
        pool.request_tx.try_send(work),
        Err(mpsc::TrySendError::Full(_))
    ));
}

pub(super) fn paint_target(
    loader: &LocalImageLoader,
    context: &egui::Context,
    path: &Path,
    viewer: &mut crate::preview_pane::ViewerState,
) -> Option<egui::Rect> {
    loader.poll(context.cumulative_frame_nr());
    let input = egui::RawInput {
        max_texture_side: Some(TEST_MAX_TEXTURE_SIDE),
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(TEST_VIEWPORT_WIDTH, TEST_VIEWPORT_HEIGHT),
        )),
        ..Default::default()
    };
    let mut response = None;
    let mut output = context.run_ui(input, |ui| {
        response = crate::preview_pane::ImageLogicOps::show_local_image(
            ui,
            path,
            TEST_IMAGE_ID,
            loader,
            Some(viewer),
            None,
            |_, _, _| {},
        );
    });
    output.textures_delta.clear();
    response
}

pub(super) fn wait_for_scheduled_repaint(
    repaint_rx: &mpsc::Receiver<(Instant, Duration)>,
    deadline: Instant,
) {
    let remaining = deadline.saturating_duration_since(Instant::now());
    assert!(!remaining.is_zero(), "no repaint before recovery deadline");
    let (requested_at, delay) = repaint_rx
        .recv_timeout(remaining)
        .expect("wait for completion or visible pending repaint");
    let mut due = requested_at + delay;
    loop {
        let until_due = due.saturating_duration_since(Instant::now());
        if until_due.is_zero() {
            return;
        }
        match repaint_rx.recv_timeout(until_due) {
            Ok((next_request, next_delay)) => due = due.min(next_request + next_delay),
            Err(mpsc::RecvTimeoutError::Timeout) => return,
            Err(error) => panic!("repaint callback disconnected: {error}"),
        }
    }
}

pub(super) fn open_fifo_writer(
    loader: &LocalImageLoader,
    context: &egui::Context,
    path: &Path,
) -> std::fs::File {
    let deadline = Instant::now() + TEST_TIMEOUT;
    loop {
        loader.poll(0);
        let _ = loader.texture(context, path, crate::theme_bridge::WHITE, TEST_IMAGE_ID);
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return writer,
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                assert!(
                    Instant::now() < deadline,
                    "worker did not open FIFO blocker"
                );
                std::thread::yield_now();
            }
            Err(error) => panic!("open FIFO writer: {error}"),
        }
    }
}

pub(super) fn make_png(path: &Path, color: [u8; RGBA_CHANNEL_COUNT], size: [u32; 2]) {
    image::RgbaImage::from_pixel(size[0], size[1], image::Rgba(color))
        .save(path)
        .expect("write real PNG fixture");
}

pub(super) fn write_png_payload(writer: &mut std::fs::File, directory: &Path) {
    let payload = directory.join("fifo-payload.png");
    make_png(&payload, FIFO_PAYLOAD_COLOR, FIFO_PAYLOAD_SIZE);
    let bytes = std::fs::read(payload).expect("read real FIFO PNG payload");
    if let Err(error) = writer.write_all(&bytes) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
}
