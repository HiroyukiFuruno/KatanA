use super::super::watcher_tests::{RGBA_CHANNELS, fixture_path, wait_ready};
use super::super::{INVALIDATION_QUEUE_CAPACITY, LocalImageLoader};
use super::WatchEvent;
use super::events::{notify_overflow, send_event};
use super::registration::{Targets, WatchTarget};
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; RGBA_CHANNELS] = [255, 0, 0, 255];
const BLUE_PIXEL: [u8; RGBA_CHANNELS] = [0, 0, 255, 255];

pub(super) fn save_png(path: &Path, pixel: [u8; RGBA_CHANNELS]) {
    image::RgbaImage::from_pixel(1, 1, image::Rgba(pixel))
        .save(path)
        .expect("fixture PNG");
}

fn target(loader: &LocalImageLoader, path: &Path) -> Targets {
    let mut targets = Targets::new();
    targets.insert(
        path.to_path_buf(),
        vec![WatchTarget {
            owner: Arc::downgrade(&loader.inner),
            generation: loader.generation(),
            requested_path: path.to_path_buf(),
        }],
    );
    targets
}

pub(super) fn fill_invalidation_queue(loader: &LocalImageLoader) {
    let generation = loader.generation();
    for _ in 0..INVALIDATION_QUEUE_CAPACITY {
        send_event(
            &loader.inner,
            WatchEvent::Changed(PathBuf::from("discarded-watcher-event.png"), generation),
        );
    }
}

pub(super) fn release_queue_slot(loader: &LocalImageLoader) {
    loader
        .inner
        .invalidation_rx
        .lock()
        .expect("invalidation receiver lock")
        .try_recv()
        .expect("queued watcher event");
}

#[test]
fn backend_overflow_notifies_registered_path_and_request_recovers() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    notify_overflow(&target(&loader, &path));
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        super::super::LocalImageStatus::Pending
    ));
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn subscriber_overflow_recovers_watched_and_lost_registered_paths() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (root, path) = fixture_path();
    let pending_path = root.path().join("pending.png");
    save_png(&path, RED_PIXEL);
    save_png(&pending_path, BLUE_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    loader
        .inner
        .watch_pending
        .lock()
        .expect("watch pending lock")
        .insert(pending_path.clone());
    fill_invalidation_queue(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Registered(pending_path.clone(), loader.generation()),
    );
    loader.poll(0);
    assert!(
        !loader
            .inner
            .watch_pending
            .lock()
            .expect("watch pending lock")
            .contains(&pending_path)
    );
    for path in [&path, &pending_path] {
        assert!(matches!(
            loader.request(path, TEST_BACKGROUND),
            super::super::LocalImageStatus::Pending
        ));
    }
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    wait_ready(&loader, &pending_path, TEST_BACKGROUND, BLUE_PIXEL);
}

#[test]
fn subscriber_overflow_preserves_dropped_registration_failure() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    send_event(
        &loader.inner,
        WatchEvent::Registered(path.clone(), loader.generation()),
    );
    for _ in 1..INVALIDATION_QUEUE_CAPACITY {
        send_event(
            &loader.inner,
            WatchEvent::Changed(
                PathBuf::from("discarded-watcher-event.png"),
                loader.generation(),
            ),
        );
    }
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "first registration failure".to_owned(),
            loader.generation(),
        ),
    );
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "latest registration failure".to_owned(),
            loader.generation(),
        ),
    );
    loader.poll(0);
    assert_eq!(
        loader.watch_error(&path).as_deref(),
        Some("latest registration failure")
    );
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        super::super::LocalImageStatus::Pending | super::super::LocalImageStatus::Ready(_)
    ));
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    send_event(
        &loader.inner,
        WatchEvent::Registered(path.clone(), loader.generation()),
    );
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}
