use super::watcher_tests::{fixture_path, wait_ready};
use super::*;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

pub(super) fn watch_state(loader: &LocalImageLoader, path: &Path) -> String {
    let revision = loader.path_revision(path);
    let watched = loader
        .inner
        .watched_paths
        .lock()
        .ok()
        .is_some_and(|paths| paths.contains(path));
    let pending = loader
        .inner
        .watch_pending
        .lock()
        .ok()
        .is_some_and(|paths| paths.contains(path));
    let error = loader.watch_error(path);
    let deferred = loader
        .inner
        .deferred_watch_results
        .lock()
        .ok()
        .and_then(|errors| errors.get(path).cloned());
    let overflow = loader
        .inner
        .invalidation_overflow
        .load(std::sync::atomic::Ordering::Acquire);
    format!(
        "revision={revision}, watched={watched}, pending={pending}, error={error:?}, deferred={deferred:?}, overflow={overflow}"
    )
}

#[test]
fn watcher_overflow_retries_registration_without_persisting_failure() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    loader
        .inner
        .invalidation_tx
        .send(WatchEvent::Overflow(path.clone(), loader.generation()))
        .expect("overflow event");
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn subscriber_overflow_retries_watched_and_pending_paths() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (root, path) = fixture_path();
    let pending_path = root.path().join("pending.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    image::RgbaImage::from_pixel(1, 1, image::Rgba(BLUE_PIXEL))
        .save(&pending_path)
        .expect("pending PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    loader
        .inner
        .watch_pending
        .lock()
        .expect("watch pending lock")
        .insert(pending_path.clone());
    loader
        .inner
        .invalidation_overflow
        .store(true, std::sync::atomic::Ordering::Release);
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    assert!(loader.watch_error(&pending_path).is_none());
    for path in [&path, &pending_path] {
        assert!(matches!(
            loader.request(path, TEST_BACKGROUND),
            LocalImageStatus::Pending
        ));
    }
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    wait_ready(&loader, &pending_path, TEST_BACKGROUND, BLUE_PIXEL);
}
