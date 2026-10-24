use super::*;
use crate::preview_pane::local_image_loader::LocalImageLoader;
use crate::preview_pane::local_image_loader::watcher::backend::Backend;
use crate::preview_pane::local_image_loader::watcher::events::{notify_event, retain_live_targets};
use crate::preview_pane::local_image_loader::watcher::registration::WatchTarget;
use crate::preview_pane::local_image_loader::watcher::tests::save_png;
use crate::preview_pane::local_image_loader::watcher_tests::wait_ready;

use super::super::{WatchCommand, WatchRequest, clear_owner};

mod fixtures;
mod lifecycle;
mod registration;

use fixtures::*;

#[test]
fn same_size_and_mtime_png_overwrite_notifies_and_reloads() {
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    let root = tempfile::tempdir().expect("image directory");
    let path = root.path().join("image.png");
    fixed_png(&path, RED);
    let before = std::fs::metadata(&path).expect("initial PNG metadata");
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, WHITE, RED);
    let mut targets = targets(&loader, &path);
    let (mut watcher, events) = content();
    watcher.sync(&HashSet::from([root.path().to_path_buf()]), &targets);
    assert_eq!(watcher.watched, HashSet::from([path.clone()]));
    fixed_png(&path, BLUE);
    let after = std::fs::metadata(&path).expect("changed PNG metadata");
    assert_eq!(before.len(), after.len());
    assert_eq!(before.modified().unwrap(), after.modified().unwrap());
    notify_event(changed(&events, &path), &mut targets);
    wait_ready(&loader, &path, WHITE, BLUE);
}

#[test]
fn new_content_baseline_rechecks_cached_image_once_without_sync_invalidation_loop() {
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    let root = tempfile::tempdir().expect("image directory");
    let directory = std::fs::canonicalize(root.path()).expect("canonical directory");
    let path = directory.join("image.png");
    fixed_png(&path, RED);
    let loader = LocalImageLoader::default();
    wait_ready(&loader, &path, WHITE, RED);
    fixed_png(&path, BLUE);
    let crate::preview_pane::local_image_loader::LocalImageStatus::Ready(cached) =
        loader.request(&path, WHITE)
    else {
        panic!("old image must remain cached before polling")
    };
    assert_eq!(cached.pixels[0].to_array(), RED);
    loader
        .inner
        .invalidation_rx
        .lock()
        .unwrap()
        .try_iter()
        .for_each(drop);
    let (mut content, _) = content();
    let mut parent = Backend::create(|_| {}).expect("parent watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    Registration::register_with_content(
        WatchRequest {
            owner: Arc::downgrade(&loader.inner),
            path: path.clone(),
            generation: loader.generation(),
        },
        &mut parent,
        &mut directories,
        &mut targets,
        &mut content,
    );
    let received = loader
        .inner
        .invalidation_rx
        .lock()
        .unwrap()
        .try_iter()
        .collect::<Vec<_>>();
    assert_eq!(
        received
            .iter()
            .filter(|event| matches!(event,
        WatchEvent::Overflow(value, _) if value == &path))
            .count(),
        1
    );
    for event in received {
        send_event(&loader.inner, event);
    }
    wait_ready(&loader, &path, WHITE, BLUE);
    for _ in 0..3 {
        content.sync(&directories, &targets);
    }
    assert!(
        !loader
            .inner
            .invalidation_rx
            .lock()
            .unwrap()
            .try_iter()
            .any(|event| matches!(event, WatchEvent::Overflow(_, _)))
    );
}
