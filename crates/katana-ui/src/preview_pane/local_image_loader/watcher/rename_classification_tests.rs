use super::super::LocalImageLoader;
use super::ImageWatcher;
use super::backend::Backend;
use super::directory_recovery::DirectoryRecovery;
use super::registration::{Targets, WatchTarget};
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn directory_name_from_any_both_events_detach_tracked_directory() {
    for mode in [RenameMode::From, RenameMode::Any, RenameMode::Both] {
        assert_mode_detaches(mode);
    }
}

fn assert_mode_detaches(mode: RenameMode) {
    let root = tempfile::tempdir().expect("fixture directory");
    let directory = root.path().join("renamed");
    std::fs::create_dir(&directory).expect("tracked directory");
    let path = directory.join("image.png");
    let loader = LocalImageLoader::default();
    let mut targets = Targets::new();
    add_target(&mut targets, &loader, &path);
    let mut watcher = Backend::create(|_| {}).expect("image watcher");
    watch_directory(&mut watcher, &directory);
    let mut watched = HashSet::from([directory.clone()]);
    DirectoryRecovery::recover(
        &Event::new(EventKind::Modify(ModifyKind::Name(mode))).add_path(directory.clone()),
        &mut watcher,
        &mut watched,
        &targets,
    );
    loader.poll(0);
    assert!(!watched.contains(&directory));
    assert!(loader.watch_error(&path).is_some());
}

fn add_target(targets: &mut Targets, loader: &LocalImageLoader, path: &std::path::Path) {
    targets.insert(
        path.to_path_buf(),
        vec![WatchTarget {
            owner: Arc::downgrade(&loader.inner),
            generation: loader.generation(),
            requested_path: path.to_path_buf(),
        }],
    );
}

fn watch_directory(watcher: &mut ImageWatcher, path: &std::path::Path) {
    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .expect("watch");
}

#[test]
fn file_name_to_and_modify_events_keep_directory_tracking() {
    let root = tempfile::tempdir().expect("fixture directory");
    let directory = root.path().join("tracked");
    std::fs::create_dir(&directory).expect("tracked directory");
    let mut watcher = Backend::create(|_| {}).expect("image watcher");
    watch_directory(&mut watcher, &directory);
    for kind in [
        EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
        EventKind::Remove(notify::event::RemoveKind::File),
    ] {
        let mut watched = HashSet::from([directory.clone()]);
        DirectoryRecovery::recover(
            &Event::new(kind).add_path(directory.join("image.png")),
            &mut watcher,
            &mut watched,
            &Targets::new(),
        );
        assert!(watched.contains(&directory));
    }
}

#[test]
fn both_event_detaches_old_registered_directory_but_keeps_new() {
    let root = tempfile::tempdir().expect("fixture directory");
    let old = root.path().join("old");
    let new = root.path().join("new");
    std::fs::create_dir_all(&old).expect("old directory");
    std::fs::create_dir_all(&new).expect("new directory");
    let mut watcher = Backend::create(|_| {}).expect("image watcher");
    watch_directory(&mut watcher, &old);
    watch_directory(&mut watcher, &new);
    let mut watched = HashSet::from([old.clone(), new.clone()]);
    DirectoryRecovery::recover(
        &Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(old.clone())
            .add_path(new.clone()),
        &mut watcher,
        &mut watched,
        &Targets::new(),
    );
    assert!(!watched.contains(&old));
    assert!(watched.contains(&new));
}
