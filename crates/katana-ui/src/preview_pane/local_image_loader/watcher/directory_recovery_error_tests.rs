use super::super::super::LocalImageLoader;
use super::super::{
    ImageWatcher,
    backend::Backend,
    registration::{Targets, WatchTarget},
};
use super::DirectoryRecovery;
use notify::{Error, ErrorKind, Event, EventKind, event::RemoveKind};
use notify::{RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct DispatchFixture {
    _root: tempfile::TempDir,
    old: PathBuf,
    nested: PathBuf,
    unrelated: PathBuf,
    exact: LocalImageLoader,
    descendant: LocalImageLoader,
    other: LocalImageLoader,
    watcher: ImageWatcher,
    watched_dirs: HashSet<PathBuf>,
    targets: Targets,
}

impl DispatchFixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("dispatch fixture root");
        let old = root.path().join("old");
        let nested = old.join("nested");
        let unrelated = root.path().join("unrelated");
        std::fs::create_dir_all(&nested).expect("nested watched directory");
        std::fs::create_dir_all(&unrelated).expect("unrelated watched directory");
        let paths = [old, nested, unrelated]
            .map(|path| std::fs::canonicalize(path).expect("canonical watched directory"));
        let mut watcher = Backend::create(|_| {}).expect("image watcher");
        let mut watched_dirs = HashSet::new();
        for path in &paths {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .expect("watch fixture directory");
            watched_dirs.insert(path.clone());
        }
        let exact = LocalImageLoader::default();
        let descendant = LocalImageLoader::default();
        let other = LocalImageLoader::default();
        let mut targets = Targets::new();
        add_target(&mut targets, &exact, &paths[0].join("exact.png"));
        add_target(&mut targets, &descendant, &paths[1].join("nested.png"));
        add_target(&mut targets, &other, &paths[2].join("other.png"));
        Self {
            _root: root,
            old: paths[0].clone(),
            nested: paths[1].clone(),
            unrelated: paths[2].clone(),
            exact,
            descendant,
            other,
            watcher,
            watched_dirs,
            targets,
        }
    }

    fn process(&mut self, result: notify::Result<Event>) {
        DirectoryRecovery::process_event(
            result,
            &mut self.watcher,
            &mut self.watched_dirs,
            &mut self.targets,
        );
    }
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

fn not_found() -> Error {
    Error::io(std::io::Error::from(std::io::ErrorKind::NotFound))
}

fn assert_error_preserved(
    error: Error,
    watched_dirs: &HashSet<PathBuf>,
    expected_kind: impl FnOnce(&ErrorKind) -> bool,
    expected_paths: &[PathBuf],
) {
    let error = DirectoryRecovery::normalize_loss(Err(error), watched_dirs)
        .expect_err("non-eligible backend errors remain errors");
    assert!(expected_kind(&error.kind));
    assert_eq!(error.paths, expected_paths);
}

#[test]
fn empty_not_found_error_becomes_removal_for_only_missing_watched_directories() {
    let root = tempfile::tempdir().expect("fixture root");
    let old = root.path().join("old");
    let nested = old.join("nested");
    let unrelated = root.path().join("unrelated");
    std::fs::create_dir_all(&nested).expect("nested watched directory");
    std::fs::create_dir_all(&unrelated).expect("unrelated watched directory");
    let watched_dirs = HashSet::from([old.clone(), nested.clone(), unrelated.clone()]);

    std::fs::remove_dir_all(&old).expect("remove watched tree");
    let event = DirectoryRecovery::normalize_loss(Err(not_found()), &watched_dirs)
        .expect("missing watched directories become a loss event");

    assert_eq!(event.kind, EventKind::Remove(RemoveKind::Any));
    assert_eq!(
        event.paths.into_iter().collect::<HashSet<_>>(),
        HashSet::from([old, nested])
    );
    assert!(unrelated.is_dir());
    assert!(watched_dirs.contains(&unrelated));
}

#[test]
fn empty_not_found_error_remains_an_error_when_all_watched_directories_exist() {
    let root = tempfile::tempdir().expect("fixture root");
    let watched_dirs = HashSet::from([root.path().to_path_buf()]);

    let error = DirectoryRecovery::normalize_loss(Err(not_found()), &watched_dirs)
        .expect_err("live watched directories do not prove a loss");

    assert!(
        matches!(error.kind, ErrorKind::Io(ref io) if io.kind() == std::io::ErrorKind::NotFound)
    );
    assert!(error.paths.is_empty());
}

#[test]
fn only_empty_path_io_not_found_is_eligible_for_loss_normalization() {
    let root = tempfile::tempdir().expect("fixture root");
    let missing = root.path().join("missing");
    let watched_dirs = HashSet::from([missing.clone()]);
    let mut pathful_not_found = not_found();
    pathful_not_found.paths.push(missing.clone());
    assert_error_preserved(
        Error::io(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
        &watched_dirs,
        |kind| matches!(kind, ErrorKind::Io(io) if io.kind() == std::io::ErrorKind::PermissionDenied),
        &[],
    );
    assert_error_preserved(
        Error::io(std::io::Error::from(std::io::ErrorKind::Other)),
        &watched_dirs,
        |kind| matches!(kind, ErrorKind::Io(io) if io.kind() == std::io::ErrorKind::Other),
        &[],
    );
    assert_error_preserved(
        Error::generic("opaque backend error"),
        &watched_dirs,
        |kind| matches!(kind, ErrorKind::Generic(message) if message == "opaque backend error"),
        &[],
    );
    assert_error_preserved(
        Error::path_not_found(),
        &watched_dirs,
        |kind| matches!(kind, ErrorKind::PathNotFound),
        &[],
    );
    assert_error_preserved(
        pathful_not_found,
        &watched_dirs,
        |kind| matches!(kind, ErrorKind::Io(io) if io.kind() == std::io::ErrorKind::NotFound),
        std::slice::from_ref(&missing),
    );
}

#[test]
fn successful_events_are_preserved() {
    let path = PathBuf::from("still-watched");
    let event = Event {
        kind: EventKind::Create(notify::event::CreateKind::Folder),
        paths: vec![path.clone()],
        attrs: Default::default(),
    };

    let normalized = DirectoryRecovery::normalize_loss(Ok(event), &HashSet::new())
        .expect("successful events pass through");

    assert_eq!(
        normalized.kind,
        EventKind::Create(notify::event::CreateKind::Folder)
    );
    assert_eq!(normalized.paths, [path]);
}

#[test]
fn repeated_anonymous_not_found_after_recovery_overflows_without_failing_live_targets() {
    let mut fixture = DispatchFixture::new();
    std::fs::remove_dir_all(&fixture.old).expect("remove nested watched root");

    fixture.process(Err(not_found()));
    assert!(!fixture.watched_dirs.contains(&fixture.old));
    assert!(!fixture.watched_dirs.contains(&fixture.nested));
    fixture.exact.poll(0);
    fixture.descendant.poll(0);
    fixture.other.poll(0);
    assert!(
        fixture
            .exact
            .watch_error(&fixture.old.join("exact.png"))
            .is_some()
    );
    assert!(
        fixture
            .descendant
            .watch_error(&fixture.nested.join("nested.png"))
            .is_some()
    );
    assert!(fixture.watched_dirs.contains(&fixture.unrelated));
    assert!(
        fixture
            .other
            .watch_error(&fixture.unrelated.join("other.png"))
            .is_none()
    );
    assert!(
        fixture
            .watched_dirs
            .iter()
            .all(|directory| directory.is_dir())
    );

    fixture.process(Err(not_found()));
    fixture.other.poll(0);
    assert!(fixture.watched_dirs.contains(&fixture.unrelated));
    assert!(
        fixture
            .other
            .watch_error(&fixture.unrelated.join("other.png"))
            .is_none()
    );
}

#[test]
fn non_anonymous_errors_continue_to_fail_targets() {
    let mut fixture = DispatchFixture::new();
    let path = fixture.unrelated.join("other.png");
    let missing = fixture._root.path().join("pathful-missing");
    let mut pathful_not_found = not_found();
    pathful_not_found.paths.push(missing);
    let errors = [
        Error::io(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
        Error::io(std::io::Error::from(std::io::ErrorKind::Other)),
        Error::generic("opaque backend error"),
        Error::path_not_found(),
        pathful_not_found,
    ];

    for error in errors {
        fixture.other.clear_watch_state(&path);
        let expected = error.to_string();
        fixture.process(Err(error));
        fixture.other.poll(0);
        assert_eq!(
            fixture.other.watch_error(&path).as_deref(),
            Some(expected.as_str())
        );
    }
}
