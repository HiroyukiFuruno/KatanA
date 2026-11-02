use super::super::WINDOWS_IMAGE_POLL_INTERVAL;
use super::DirectoryWatcher;
use super::state::DirectoryMetadata;
use notify::event::RemoveKind;
use notify::{Config, Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(super) const EVENT_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) type Events = mpsc::Receiver<notify::Result<Event>>;

pub(super) fn watcher() -> (DirectoryWatcher, Events) {
    let (sender, events) = mpsc::channel();
    let watcher = DirectoryWatcher::new(
        sender,
        Config::default().with_poll_interval(WINDOWS_IMAGE_POLL_INTERVAL),
    )
    .expect("directory metadata watcher");
    (watcher, events)
}

fn receive_loss(events: &Events, expected: &Path) {
    let event = events
        .recv_timeout(EVENT_TIMEOUT)
        .expect("directory loss notification")
        .expect("directory loss event");
    assert_eq!(event.kind, EventKind::Remove(RemoveKind::Folder));
    assert_eq!(event.paths, [expected.to_path_buf()]);
}

#[test]
fn registered_directory_rename_detaches_old_and_nested_without_listing_children() {
    let root = tempfile::tempdir().expect("directory fixture");
    let old = root.path().join("old");
    let nested = old.join("nested");
    let other = root.path().join("unrelated");
    std::fs::create_dir_all(&nested).expect("nested directory");
    std::fs::create_dir(&other).expect("unrelated directory");
    let (mut watcher, events) = watcher();
    for path in [&old, &nested, &other] {
        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .expect("directory registration");
    }
    std::fs::rename(&old, root.path().join("renamed")).expect("immediate directory rename");
    assert_rename_losses(&events, old, nested);
    assert!(other.is_dir());
}

fn assert_rename_losses(events: &Events, old: PathBuf, nested: PathBuf) {
    let deadline = Instant::now() + EVENT_TIMEOUT;
    let mut removed = HashSet::new();
    while removed.len() < 2 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let event = events
            .recv_timeout(remaining)
            .expect("rename loss")
            .expect("loss event");
        assert_eq!(event.kind, EventKind::Remove(RemoveKind::Folder));
        removed.extend(event.paths);
    }
    assert_eq!(removed, HashSet::from([old, nested]));
}

#[test]
fn removed_directory_emits_folder_loss() {
    let root = tempfile::tempdir().expect("directory fixture");
    let path = root.path().join("removed");
    std::fs::create_dir(&path).expect("watched directory");
    let (mut watcher, events) = watcher();
    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("directory registration");
    std::fs::remove_dir(&path).expect("remove watched directory");
    receive_loss(&events, &path);
}

#[test]
fn directory_replaced_by_file_emits_folder_loss() {
    let root = tempfile::tempdir().expect("directory fixture");
    let path = root.path().join("replaced");
    std::fs::create_dir(&path).expect("watched directory");
    let (mut watcher, events) = watcher();
    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("directory registration");
    std::fs::remove_dir(&path).expect("remove watched directory");
    std::fs::write(&path, b"replacement").expect("replace directory by file");
    receive_loss(&events, &path);
    assert!(
        DirectoryMetadata::sample(&path)
            .expect("replacement loss")
            .is_ok()
    );
}

#[test]
fn non_directory_ancestor_emits_folder_loss() {
    let root = tempfile::tempdir().expect("directory fixture");
    let ancestor = root.path().join("ancestor");
    let nested = ancestor.join("nested");
    std::fs::create_dir_all(&nested).expect("nested directory");
    let (mut watcher, events) = watcher();
    watcher
        .watch(&nested, RecursiveMode::NonRecursive)
        .expect("directory registration");
    std::fs::remove_dir(&nested).expect("remove nested directory");
    std::fs::remove_dir(&ancestor).expect("remove parent directory");
    std::fs::write(&ancestor, b"replacement").expect("replace parent by file");
    receive_loss(&events, &nested);
}

#[test]
fn registration_requires_a_directory_and_rejects_recursive_mode() {
    let root = tempfile::tempdir().expect("directory fixture");
    let file = root.path().join("file");
    std::fs::write(&file, b"file").expect("non-directory input");
    let (mut watcher, _) = watcher();
    assert!(
        watcher
            .watch(root.path(), RecursiveMode::NonRecursive)
            .is_ok()
    );
    assert!(
        watcher
            .watch(root.path(), RecursiveMode::Recursive)
            .is_err()
    );
    let error = watcher
        .watch(&file, RecursiveMode::NonRecursive)
        .expect_err("file rejected");
    assert!(
        matches!(error.kind, notify::ErrorKind::Io(error) if error.kind() == std::io::ErrorKind::NotADirectory)
    );
    assert!(
        watcher
            .watch(&root.path().join("missing"), RecursiveMode::NonRecursive)
            .is_err()
    );
}

#[test]
fn permission_and_unexpected_metadata_errors_remain_path_scoped_errors() {
    let path = PathBuf::from("watched");
    for kind in [
        std::io::ErrorKind::PermissionDenied,
        std::io::ErrorKind::Other,
    ] {
        let result = DirectoryMetadata::classify(&path, Err(std::io::Error::from(kind)));
        let error = result
            .expect("metadata error event")
            .expect_err("error preserved");
        assert_eq!(error.paths.as_slice(), std::slice::from_ref(&path));
        assert!(matches!(error.kind, notify::ErrorKind::Io(error) if error.kind() == kind));
    }
}
