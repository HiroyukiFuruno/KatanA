use super::super::{LocalImageLoader, watcher_tests::fixture_path};
use super::events::notify_targets;
use super::registration::{Targets, WatchTarget};
use super::tests::save_png;
use std::path::Path;
use std::sync::Arc;

const RED_PIXEL: [u8; super::super::watcher_tests::RGBA_CHANNELS] = [255, 0, 0, 255];

fn add_target(targets: &mut Targets, loader: &LocalImageLoader, path: &Path) {
    targets.insert(
        path.to_path_buf(),
        vec![WatchTarget {
            owner: Arc::downgrade(&loader.inner),
            generation: loader.generation(),
            requested_path: path.to_path_buf(),
        }],
    );
}

#[test]
fn deleted_parent_event_does_not_fail_unrelated_live_image() {
    assert_deleted_event_is_scoped(false);
}

#[test]
fn deleted_directory_event_still_invalidates_its_image_descendants() {
    assert_deleted_event_is_scoped(true);
}

fn assert_deleted_event_is_scoped(directory_event: bool) {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let (removed_root, removed_path) = fixture_path();
    let (_live_root, live_path) = fixture_path();
    save_png(&removed_path, RED_PIXEL);
    save_png(&live_path, RED_PIXEL);
    let removed_path = std::fs::canonicalize(removed_path).expect("removed image path");
    let live_path = std::fs::canonicalize(live_path).expect("live image path");
    let removed = LocalImageLoader::default();
    let live = LocalImageLoader::default();
    let mut targets = Targets::new();
    add_target(&mut targets, &removed, &removed_path);
    add_target(&mut targets, &live, &live_path);
    let event_path = if directory_event {
        removed_path.parent().expect("removed parent").to_path_buf()
    } else {
        removed_path.clone()
    };
    drop(removed_root);
    notify_targets(
        notify::EventKind::Remove(notify::event::RemoveKind::Any),
        &[event_path],
        &mut targets,
    );
    removed.poll(0);
    live.poll(0);
    assert!(
        live.watch_error(&live_path).is_none(),
        "{:?}",
        live.watch_error(&live_path)
    );
    assert_eq!(live.path_revision(&live_path), 0);
    assert_eq!(removed.path_revision(&removed_path), 1);
    assert!(removed.watch_error(&removed_path).is_none());
}
