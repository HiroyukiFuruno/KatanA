use super::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const WAIT_SECONDS: u64 = 5;
const REQUEST_REPETITIONS: usize = 100;
const RETRY_REPAINT_LIMIT: Duration = Duration::from_secs(2);

fn wait_ready_and_registered(loader: &LocalImageLoader, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        /* WHY: decode復旧と非同期watch登録は独立して完了するため、同じ期限内で両方の契約を確認する。 */
        if let LocalImageStatus::Ready(image) = loader.request(path, TEST_BACKGROUND)
            && image.pixels.first().map(egui::Color32::to_array) == Some(RED_PIXEL)
            && loader.watch_error(path).is_none()
        {
            return;
        }
        std::thread::yield_now();
    }
    let state = super::watcher_overflow_tests::watch_state(loader, path);
    panic!("image and watch registration did not recover: {state}");
}

#[test]
fn missing_parent_recovers_after_directory_and_png_are_created() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let root = tempfile::tempdir().expect("fixture directory");
    let path = root.path().join("created/image.png");
    let loader = LocalImageLoader::default();
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        if matches!(
            loader.request(&path, TEST_BACKGROUND),
            LocalImageStatus::Failed(_)
        ) {
            break;
        }
        frame += 1;
        std::thread::yield_now();
    }
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Failed(_)
    ));
    assert!(loader.watch_error(&path).is_some());
    assert!(
        loader
            .inner
            .watch_errors
            .lock()
            .expect("watch failures lock")
            .contains_key(&path)
    );
    std::fs::create_dir_all(path.parent().expect("parent directory")).expect("create parent");
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("created PNG");
    wait_ready_and_registered(&loader, &path);
    assert!(loader.watch_error(&path).is_none());
}

#[test]
fn retry_keeps_failure_visible_and_schedules_repaint() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let root = tempfile::tempdir().expect("fixture directory");
    let path = root.path().join("missing/image.png");
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        if matches!(
            loader.request(&path, TEST_BACKGROUND),
            LocalImageStatus::Failed(_)
        ) {
            break;
        }
        frame += 1;
        std::thread::yield_now();
    }
    let status = loader.request(&path, TEST_BACKGROUND);
    assert!(matches!(status, LocalImageStatus::Failed(_)));
    let (deadline, attempts) = {
        let errors = loader
            .inner
            .watch_errors
            .lock()
            .expect("watch failures lock");
        let failure = errors.get(&path).expect("watch failure");
        (failure.deadline, failure.attempts)
    };
    context.request_repaint_after(Duration::from_secs(WAIT_SECONDS));
    let callback_delay = Arc::new(Mutex::new(None));
    let callback_delay_clone = Arc::clone(&callback_delay);
    context.set_request_repaint_callback(move |info| {
        if info.delay > Duration::ZERO {
            *callback_delay_clone.lock().expect("callback delay lock") = Some(info.delay);
        }
    });
    loader.set_repaint_context(&context);
    for _ in 0..REQUEST_REPETITIONS {
        assert!(matches!(
            loader.request(&path, TEST_BACKGROUND),
            LocalImageStatus::Failed(_)
        ));
    }
    let failures = loader
        .inner
        .watch_errors
        .lock()
        .expect("watch failures lock");
    assert!(failures[&path].deadline >= deadline);
    assert!(failures[&path].attempts >= attempts);
    drop(failures);
    let delay = callback_delay
        .lock()
        .expect("callback delay lock")
        .expect("delayed repaint callback");
    assert!(delay > Duration::ZERO && delay <= RETRY_REPAINT_LIMIT);
    assert!(
        loader
            .inner
            .watch_pending
            .lock()
            .expect("pending watches lock")
            .len()
            <= 1
    );
}
