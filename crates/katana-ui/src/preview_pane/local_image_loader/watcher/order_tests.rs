use super::super::watcher_tests::{RGBA_CHANNELS, fixture_path, wait_ready};
use super::super::{LocalImageLoader, LocalImageStatus};
use super::WatchEvent;
use super::events::send_event;
use super::tests::{fill_invalidation_queue, release_queue_slot, save_png};

const TEST_BACKGROUND: eframe::egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; RGBA_CHANNELS] = [255, 0, 0, 255];

#[test]
fn lost_registered_event_clears_an_already_applied_failure() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "prior failure".to_owned(),
            loader.generation(),
        ),
    );
    loader.poll(0);
    assert_eq!(loader.watch_error(&path).as_deref(), Some("prior failure"));
    fill_invalidation_queue(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Registered(path.clone(), loader.generation()),
    );
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn later_registered_event_clears_deferred_failure() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    fill_invalidation_queue(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "earlier failure".to_owned(),
            loader.generation(),
        ),
    );
    release_queue_slot(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Registered(path.clone(), loader.generation()),
    );
    loader.poll(0);
    assert!(loader.watch_error(&path).is_none());
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn later_failed_event_replaces_deferred_failure_after_capacity_returns() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    fill_invalidation_queue(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "earlier failure".to_owned(),
            loader.generation(),
        ),
    );
    release_queue_slot(&loader);
    send_event(
        &loader.inner,
        WatchEvent::Failed(
            path.clone(),
            "latest failure".to_owned(),
            loader.generation(),
        ),
    );
    loader.poll(0);
    assert_eq!(loader.watch_error(&path).as_deref(), Some("latest failure"));
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Failed(_)
    ));
}
