use super::super::watcher_tests::{RGBA_CHANNELS, fixture_path, wait_ready};
use super::super::{LocalImageLoader, LocalImageStatus};
use super::registration::Registration;
use super::tests::save_png;
use super::{Targets, WatchRequest, unwatch_unused_dirs};
use notify::{Config, RecommendedWatcher, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

const TEST_BACKGROUND: eframe::egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; RGBA_CHANNELS] = [255, 0, 0, 255];

fn register(
    loader: &LocalImageLoader,
    path: &Path,
    watcher: &mut RecommendedWatcher,
    directories: &mut HashSet<std::path::PathBuf>,
    targets: &mut Targets,
) {
    Registration::register(
        WatchRequest {
            owner: Arc::downgrade(&loader.inner),
            path: path.to_path_buf(),
            generation: loader.generation(),
        },
        watcher,
        directories,
        targets,
    );
    loader.poll(0);
}

#[test]
fn adding_a_directory_invalidates_existing_subscription_after_backend_restart() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_first, first_path) = fixture_path();
    let (_second, second_path) = fixture_path();
    save_png(&first_path, RED_PIXEL);
    save_png(&second_path, RED_PIXEL);
    let first = LocalImageLoader::default();
    let second = LocalImageLoader::default();
    wait_ready(&first, &first_path, TEST_BACKGROUND, RED_PIXEL);
    let mut watcher = RecommendedWatcher::new(|_| {}, Config::default()).expect("native watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    register(
        &first,
        &first_path,
        &mut watcher,
        &mut directories,
        &mut targets,
    );
    let revision = first.path_revision(&first_path);
    register(
        &second,
        &second_path,
        &mut watcher,
        &mut directories,
        &mut targets,
    );
    first.poll(0);
    assert!(first.path_revision(&first_path) > revision);
    assert!(first.watch_error(&first_path).is_none());
    assert!(matches!(
        first.request(&first_path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    wait_ready(&first, &first_path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn removing_a_directory_invalidates_remaining_subscription_after_backend_restart() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_first, first_path) = fixture_path();
    let (_second, second_path) = fixture_path();
    save_png(&first_path, RED_PIXEL);
    save_png(&second_path, RED_PIXEL);
    let first = LocalImageLoader::default();
    let second = LocalImageLoader::default();
    wait_ready(&first, &first_path, TEST_BACKGROUND, RED_PIXEL);
    let mut watcher = RecommendedWatcher::new(|_| {}, Config::default()).expect("native watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    register(
        &first,
        &first_path,
        &mut watcher,
        &mut directories,
        &mut targets,
    );
    register(
        &second,
        &second_path,
        &mut watcher,
        &mut directories,
        &mut targets,
    );
    first.poll(0);
    wait_ready(&first, &first_path, TEST_BACKGROUND, RED_PIXEL);
    let revision = first.path_revision(&first_path);
    let second_parent =
        std::fs::canonicalize(second_path.parent().expect("parent")).expect("canonical parent");
    targets.retain(|_, owners| {
        owners.retain(|target| !target.owner.ptr_eq(&Arc::downgrade(&second.inner)));
        !owners.is_empty()
    });
    unwatch_unused_dirs(&mut watcher, &mut directories, &targets);
    first.poll(0);
    assert!(!directories.contains(&second_parent));
    assert!(first.path_revision(&first_path) > revision);
    assert!(first.watch_error(&first_path).is_none());
    wait_ready(&first, &first_path, TEST_BACKGROUND, RED_PIXEL);
}

#[test]
fn backend_rescan_flag_invalidates_registered_image_without_a_path() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (_root, path) = fixture_path();
    save_png(&path, RED_PIXEL);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    let mut watcher = RecommendedWatcher::new(|_| {}, Config::default()).expect("native watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    register(&loader, &path, &mut watcher, &mut directories, &mut targets);
    let revision = loader.path_revision(&path);
    super::events::notify_event(
        notify::Event::new(notify::EventKind::Other).set_flag(notify::event::Flag::Rescan),
        &mut targets,
    );
    loader.poll(0);
    assert!(loader.path_revision(&path) > revision);
    assert!(loader.watch_error(&path).is_none());
    wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
}
