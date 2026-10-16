use super::super::LocalImageLoader;
use super::directory_recovery::DirectoryRecovery;
use super::registration::{Targets, WatchTarget};
use notify::event::{ModifyKind, RenameMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

const EVENT_TIMEOUT: Duration = Duration::from_secs(5);
const OBSERVED_EVENT_LIMIT: usize = 8;
type FixturePaths = (tempfile::TempDir, PathBuf, PathBuf, PathBuf);
type LoaderSet = (
    LocalImageLoader,
    LocalImageLoader,
    LocalImageLoader,
    Targets,
);
type WatcherEvents = mpsc::Receiver<Result<Event, notify::Error>>;

struct RenameFixture {
    _root: tempfile::TempDir,
    old: PathBuf,
    nested: PathBuf,
    unrelated: PathBuf,
    watched_dirs: HashSet<PathBuf>,
    targets: Targets,
    exact: LocalImageLoader,
    descendant: LocalImageLoader,
    other: LocalImageLoader,
    watcher: RecommendedWatcher,
    events: mpsc::Receiver<Result<Event, notify::Error>>,
}

impl RenameFixture {
    fn new() -> Self {
        let (root, old, nested, unrelated) = Self::paths();
        let (exact, descendant, other, targets) = Self::loaders(&old, &nested, &unrelated);
        let (watcher, events) = Self::watcher(&root, &old, &nested, &unrelated);
        let watched_dirs = HashSet::from([old.clone(), nested.clone(), unrelated.clone()]);
        mark_watched(&exact, &old.join("exact.png"));
        mark_watched(&descendant, &nested.join("nested.png"));
        mark_watched(&other, &unrelated.join("other.png"));
        Self {
            _root: root,
            old,
            nested,
            unrelated,
            watched_dirs,
            targets,
            exact,
            descendant,
            other,
            watcher,
            events,
        }
    }

    fn paths() -> FixturePaths {
        let root = tempfile::tempdir().expect("rename fixture");
        let old = root.path().join("old");
        let nested = old.join("nested");
        let unrelated = root.path().join("unrelated");
        std::fs::create_dir_all(&nested).expect("nested directory");
        std::fs::create_dir_all(&unrelated).expect("unrelated directory");
        (
            root,
            std::fs::canonicalize(old).expect("canonical old"),
            std::fs::canonicalize(nested).expect("canonical nested"),
            std::fs::canonicalize(unrelated).expect("canonical unrelated"),
        )
    }

    fn loaders(old: &Path, nested: &Path, unrelated: &Path) -> LoaderSet {
        let exact = LocalImageLoader::default();
        let descendant = LocalImageLoader::default();
        let other = LocalImageLoader::default();
        let mut targets = Targets::new();
        add_target(&mut targets, &exact, &old.join("exact.png"));
        add_target(&mut targets, &descendant, &nested.join("nested.png"));
        add_target(&mut targets, &other, &unrelated.join("other.png"));
        (exact, descendant, other, targets)
    }

    fn watcher(
        root: &tempfile::TempDir,
        old: &Path,
        nested: &Path,
        unrelated: &Path,
    ) -> (RecommendedWatcher, WatcherEvents) {
        let (tx, events) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                let _ = tx.send(event);
            },
            Config::default(),
        )
        .expect("native watcher");
        watcher
            .watch(root.path(), RecursiveMode::Recursive)
            .expect("root watch");
        for path in [old, nested, unrelated] {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .expect("target watch");
        }
        (watcher, events)
    }

    fn receive_rename(&self) -> Event {
        let deadline = Instant::now() + EVENT_TIMEOUT;
        let mut observed = Vec::new();
        loop {
            let event = match self
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(Ok(event)) => event,
                Ok(Err(error)) => panic!("native watcher error: {error}; observed={observed:?}"),
                Err(error) => panic!("native rename event {error}; observed={observed:?}"),
            };
            if is_directory_rename(&event) && event.paths.iter().any(|path| path == &self.old) {
                return event;
            }
            observed.push(format!("{:?}:{:?}", event.kind, event.paths));
            if observed.len() > OBSERVED_EVENT_LIMIT {
                observed.remove(0);
            }
        }
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

fn mark_watched(loader: &LocalImageLoader, path: &Path) {
    loader
        .inner
        .watched_paths
        .lock()
        .expect("watched paths")
        .insert(path.to_path_buf());
}

fn is_directory_rename(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Modify(ModifyKind::Name(
            RenameMode::From | RenameMode::Any | RenameMode::Both
        ))
    )
}

fn recover(fixture: &mut RenameFixture, event: Event) {
    DirectoryRecovery::recover(
        &event,
        &mut fixture.watcher,
        &mut fixture.watched_dirs,
        &fixture.targets,
    );
}

fn assert_failed(loader: &LocalImageLoader, path: &Path) {
    assert!(loader.watch_error(path).is_some());
}

fn assert_healthy(loader: &LocalImageLoader, path: &Path) {
    assert!(loader.watch_error(path).is_none());
}

#[test]
fn native_directory_rename_detaches_exact_descendants_and_keeps_unrelated_watch() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let mut fixture = RenameFixture::new();
    let renamed = fixture.old.with_file_name("new");
    std::fs::rename(&fixture.old, &renamed).expect("directory rename");
    let event = fixture.receive_rename();
    recover(&mut fixture, event);
    fixture.exact.poll(0);
    fixture.descendant.poll(0);
    fixture.other.poll(0);
    assert!(!fixture.watched_dirs.contains(&fixture.old));
    assert!(!fixture.watched_dirs.contains(&fixture.nested));
    assert!(fixture.watched_dirs.contains(&fixture.unrelated));
    assert_failed(&fixture.exact, &fixture.old.join("exact.png"));
    assert_failed(&fixture.descendant, &fixture.nested.join("nested.png"));
    assert_healthy(&fixture.other, &fixture.unrelated.join("other.png"));
}
