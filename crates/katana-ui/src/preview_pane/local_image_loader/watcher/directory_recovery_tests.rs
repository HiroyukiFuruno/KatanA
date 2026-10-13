use super::super::{LocalImageLoader, watcher_tests::RGBA_CHANNELS};
use super::WatchRequest;
use super::directory_recovery::DirectoryRecovery;
use super::events::notify_event;
use super::registration::{Registration, Targets, WatchTarget};
use super::tests::save_png;
use notify::{Config, EventKind, RecommendedWatcher, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

const RED_PIXEL: [u8; RGBA_CHANNELS] = [255, 0, 0, 255];
const BLUE_PIXEL: [u8; RGBA_CHANNELS] = [0, 0, 255, 255];
const NATIVE_EVENT_WAIT: Duration = Duration::from_secs(5);

fn register(
    loader: &LocalImageLoader,
    path: &Path,
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &mut Targets,
) {
    Registration::register(
        WatchRequest {
            owner: Arc::downgrade(&loader.inner),
            path: path.to_path_buf(),
            generation: loader.generation(),
        },
        watcher,
        watched_dirs,
        targets,
    );
    loader.poll(0);
}

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

fn receive_removed_event(
    events: &mpsc::Receiver<Result<notify::Event, notify::Error>>,
    parent: &Path,
) -> notify::Event {
    let deadline = Instant::now() + NATIVE_EVENT_WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("native parent removal event");
        let event = event.expect("native watcher event");
        if matches!(event.kind, EventKind::Remove(_))
            && event.paths.iter().any(|path| path == parent)
        {
            return event;
        }
    }
}

fn receive_image_event(
    events: &mpsc::Receiver<Result<notify::Event, notify::Error>>,
    image_path: &Path,
) -> notify::Event {
    let deadline = Instant::now() + NATIVE_EVENT_WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("native image change event");
        let event = event.expect("native watcher event");
        if event.paths.iter().any(|path| path == image_path) {
            return event;
        }
    }
}

#[test]
fn recreated_watched_parent_receives_later_overwrite() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let root = tempfile::tempdir().expect("fixture directory");
    let parent = root.path().join("watched");
    std::fs::create_dir_all(&parent).expect("watched parent");
    let image_path = parent.join("image.png");
    save_png(&image_path, RED_PIXEL);
    let live_root = tempfile::tempdir().expect("live fixture directory");
    let live_path = live_root.path().join("live.png");
    save_png(&live_path, RED_PIXEL);
    let removed = LocalImageLoader::default();
    let live = LocalImageLoader::default();
    let (event_tx, event_rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            let _ = event_tx.send(event);
        },
        Config::default(),
    )
    .expect("native watcher");
    let mut watched_dirs = HashSet::new();
    let mut targets = Targets::new();
    let image_path = std::fs::canonicalize(&image_path).expect("image path");
    let live_path = std::fs::canonicalize(&live_path).expect("live path");
    register(
        &removed,
        &image_path,
        &mut watcher,
        &mut watched_dirs,
        &mut targets,
    );
    add_target(&mut targets, &live, &live_path);
    let live_parent = live_path.parent().expect("live parent").to_path_buf();
    watcher
        .watch(&live_parent, notify::RecursiveMode::NonRecursive)
        .expect("live watch");
    watched_dirs.insert(live_parent.clone());
    let removed_parent = image_path.parent().expect("image parent").to_path_buf();
    assert!(watched_dirs.contains(&removed_parent));

    std::fs::remove_dir_all(&removed_parent).expect("remove watched parent");
    let event = receive_removed_event(&event_rx, &removed_parent);
    DirectoryRecovery::recover(&event, &mut watcher, &mut watched_dirs, &targets);
    notify_event(event, &mut targets);
    removed.poll(0);
    live.poll(0);
    let first_revision = removed.path_revision(&image_path);
    assert!(first_revision > 0);
    assert!(!watched_dirs.contains(&removed_parent));
    assert!(removed.watch_error(&image_path).is_some());
    assert_eq!(live.path_revision(&live_path), 0);
    assert!(live.watch_error(&live_path).is_none());

    std::fs::create_dir_all(&removed_parent).expect("recreate watched parent");
    save_png(&image_path, RED_PIXEL);
    register(
        &removed,
        &image_path,
        &mut watcher,
        &mut watched_dirs,
        &mut targets,
    );
    while event_rx.try_recv().is_ok() {}
    save_png(&image_path, BLUE_PIXEL);
    let event = receive_image_event(&event_rx, &image_path);
    notify_event(event, &mut targets);
    removed.poll(0);
    assert!(removed.path_revision(&image_path) > first_revision);
    assert!(targets.contains_key(&live_path));
    assert!(removed.watch_error(&image_path).is_none());
    assert!(live.watch_error(&live_path).is_none());
}
