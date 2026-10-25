use super::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const WAIT_SECONDS: u64 = 5;
pub(super) const RGBA_CHANNELS: usize = 4;
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

pub(super) fn fixture_path() -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::Builder::new()
        .prefix("katana-local-image-watch-")
        .tempdir_in(".")
        .expect("fixture directory");
    std::fs::create_dir_all(root.path().join("nested")).expect("nested directory");
    let current = std::env::current_dir().expect("current directory");
    let relative_root = root.path().strip_prefix(current).expect("relative fixture");
    let path = relative_root.join("nested/../image.png");
    (root, path)
}

pub(super) fn wait_ready(
    loader: &LocalImageLoader,
    path: &Path,
    background: egui::Color32,
    expected: [u8; RGBA_CHANNELS],
) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        if let LocalImageStatus::Ready(image) = loader.request(path, background)
            && image.pixels.first().map(egui::Color32::to_array) == Some(expected)
        {
            return;
        }
        frame += 1;
        std::thread::yield_now();
    }
    let state = super::watcher_overflow_tests::watch_state(loader, path);
    panic!("watcher did not publish expected image: {expected:?}; {state}");
}

fn wait_revision(loader: &LocalImageLoader, path: &Path, previous: u64) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        if loader.path_revision(path) != previous {
            return;
        }
        frame += 1;
        std::thread::yield_now();
    }
    let state = super::watcher_overflow_tests::watch_state(loader, path);
    panic!("watcher did not publish revision: {state}");
}

#[test]
fn atomic_replace_reloads_relative_path_without_touching_other_background() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (root, path) = fixture_path();
    let other_path = root.path().join("other.png");
    let sentinel_path = root.path().join("sentinel.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("initial PNG");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 255, 255]))
        .save(&other_path)
        .expect("unrelated PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    wait_ready(&loader, &path, crate::theme_bridge::BLACK, RED_PIXEL);
    wait_ready(&loader, &other_path, TEST_BACKGROUND, BLUE_PIXEL);
    let LocalImageStatus::Ready(other_image) = loader.request(&other_path, TEST_BACKGROUND) else {
        panic!("unrelated image must be cached");
    };
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 255, 255]))
        .save(&sentinel_path)
        .expect("sentinel PNG");
    wait_ready(&loader, &sentinel_path, TEST_BACKGROUND, BLUE_PIXEL);
    let revision = loader.path_revision(&path);
    let sentinel_revision = loader.path_revision(&sentinel_path);
    let _ = std::fs::read(&path).expect("access event source");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 255, 255, 255]))
        .save(&sentinel_path)
        .expect("sentinel change");
    wait_revision(&loader, &sentinel_path, sentinel_revision);
    assert_eq!(loader.path_revision(&path), revision);

    let replacement = root.path().join("replacement.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 255, 0, 255]))
        .save(&replacement)
        .expect("replacement PNG");
    std::fs::rename(&replacement, root.path().join("image.png")).expect("atomic replacement");
    wait_ready(&loader, &path, TEST_BACKGROUND, GREEN_PIXEL);
    wait_ready(&loader, &path, crate::theme_bridge::BLACK, GREEN_PIXEL);
    let LocalImageStatus::Ready(unchanged) = loader.request(&other_path, TEST_BACKGROUND) else {
        panic!("unrelated image must remain cached");
    };
    assert!(Arc::ptr_eq(&unchanged, &other_image));
}

#[test]
fn reset_drops_old_watch_generation_before_rerequest() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    loader
        .inner
        .deferred_watch_results
        .lock()
        .expect("deferred errors lock")
        .insert(
            path.clone(),
            (loader.generation(), Err("stale failure".to_owned())),
        );
    loader.reset();
    assert!(
        loader
            .inner
            .deferred_watch_results
            .lock()
            .expect("deferred errors lock")
            .is_empty()
    );
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 255, 0, 255]))
        .save(&path)
        .expect("reset PNG");
    wait_ready(&loader, &path, TEST_BACKGROUND, GREEN_PIXEL);
}

#[test]
fn successful_registration_clears_prior_watch_failure() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    let generation = loader.generation();
    loader
        .inner
        .invalidation_tx
        .send(WatchEvent::Failed(
            path.clone(),
            "watch registration failed".to_owned(),
            generation,
        ))
        .expect("failure event");
    loader.poll(0);
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Ready(_)
    ));
    loader
        .inner
        .invalidation_overflow
        .store(true, std::sync::atomic::Ordering::Release);
    loader.poll(0);
    assert!(loader.watch_error(&path).is_some());
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Ready(_)
    ));
    loader
        .inner
        .invalidation_tx
        .send(WatchEvent::Registered(path.clone(), generation))
        .expect("registration event");
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn watch_registration_failure_does_not_block_readable_image_request() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    let generation = loader.generation();
    loader
        .inner
        .invalidation_tx
        .send(WatchEvent::Failed(
            path.clone(),
            "watch registration failed".to_owned(),
            generation,
        ))
        .expect("failure event");
    loader.poll(0);

    loader
        .inner
        .watch_errors
        .lock()
        .expect("watch failures lock")
        .get_mut(&path)
        .expect("watch failure")
        .deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS * 2);

    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    assert!(loader.watch_error(&path).is_some());
}

#[test]
fn repeated_watch_failures_preserve_ready_until_reregistered() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    let LocalImageStatus::Ready(initial) = loader.request(&path, TEST_BACKGROUND) else {
        panic!("initial image must be ready");
    };
    let generation = loader.generation();
    let old_revision = loader.path_revision(&path);
    for error in ["first failure", "second failure"] {
        loader
            .inner
            .invalidation_tx
            .send(WatchEvent::Failed(
                path.clone(),
                error.to_owned(),
                generation,
            ))
            .expect("failure event");
        loader.poll(0);
        let LocalImageStatus::Ready(current) = loader.request(&path, TEST_BACKGROUND) else {
            panic!("watch failure must not hide ready image");
        };
        assert!(Arc::ptr_eq(&initial, &current));
    }
    assert_eq!(loader.watch_error(&path).as_deref(), Some("second failure"));

    image::RgbaImage::from_pixel(1, 1, image::Rgba(GREEN_PIXEL))
        .save(&path)
        .expect("updated PNG");
    loader
        .inner
        .invalidation_tx
        .send(WatchEvent::Registered(path.clone(), generation))
        .expect("registration event");
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    assert!(loader.path_revision(&path) > old_revision);
    loader
        .inner
        .result_tx
        .send(ResultMessage {
            key: RequestKey {
                path: path.clone(),
                background: TEST_BACKGROUND,
            },
            generation,
            revision: old_revision,
            image: Ok(egui::ColorImage::new(
                [1, 1],
                vec![egui::Color32::from_rgba_unmultiplied(255, 0, 0, 255)],
            )),
        })
        .expect("stale result");
    loader.poll(0);
    wait_ready(&loader, &path, TEST_BACKGROUND, GREEN_PIXEL);
}
