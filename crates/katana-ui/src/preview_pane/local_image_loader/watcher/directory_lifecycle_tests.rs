use super::super::LocalImageLoader;
use super::super::watcher::tests::save_png;
use super::directory_recovery::DirectoryRecovery;
use super::events::notify_error;
use super::registration::{Targets, WatchTarget};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];

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

fn add_watched_path(loader: &LocalImageLoader, path: &Path) {
    loader
        .inner
        .watched_paths
        .lock()
        .expect("watched paths lock")
        .insert(path.to_path_buf());
}

fn register_directory(
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    directory: &Path,
) {
    watcher
        .watch(directory, RecursiveMode::NonRecursive)
        .expect("native directory watch");
    watched_dirs.insert(directory.to_path_buf());
}

struct Fixture {
    _root: tempfile::TempDir,
    _live_root: tempfile::TempDir,
    exact_path: PathBuf,
    descendant_path: PathBuf,
    unrelated_path: PathBuf,
    exact_loader: LocalImageLoader,
    descendant_loader: LocalImageLoader,
    unrelated_loader: LocalImageLoader,
    watcher: RecommendedWatcher,
    watched_dirs: HashSet<PathBuf>,
    targets: Targets,
}

impl Fixture {
    fn paths() -> (
        tempfile::TempDir,
        tempfile::TempDir,
        PathBuf,
        PathBuf,
        PathBuf,
    ) {
        let root = tempfile::tempdir().expect("fixture directory");
        let exact = root.path().join("exact");
        let descendant = exact.join("descendant");
        let live_root = tempfile::tempdir().expect("unrelated fixture directory");
        let unrelated = live_root.path().join("unrelated");
        std::fs::create_dir_all(&descendant).expect("descendant directory");
        std::fs::create_dir_all(&unrelated).expect("unrelated directory");
        let exact_path = exact.join("exact.png");
        let descendant_path = descendant.join("descendant.png");
        let unrelated_path = unrelated.join("unrelated.png");
        for path in [&exact_path, &descendant_path, &unrelated_path] {
            save_png(path, RED_PIXEL);
        }
        (root, live_root, exact_path, descendant_path, unrelated_path)
    }

    fn new() -> Self {
        let (root, live_root, exact, descendant, unrelated) = Self::paths();
        let exact_path = std::fs::canonicalize(exact).expect("exact path");
        let descendant_path = std::fs::canonicalize(descendant).expect("descendant path");
        let unrelated_path = std::fs::canonicalize(unrelated).expect("unrelated path");
        let exact_loader = LocalImageLoader::default();
        let descendant_loader = LocalImageLoader::default();
        let unrelated_loader = LocalImageLoader::default();
        let mut targets = Targets::new();
        add_target(&mut targets, &exact_loader, &exact_path);
        add_target(&mut targets, &descendant_loader, &descendant_path);
        add_target(&mut targets, &unrelated_loader, &unrelated_path);
        let watcher = RecommendedWatcher::new(|_| {}, Config::default()).expect("native watcher");
        let mut fixture = Self {
            _root: root,
            _live_root: live_root,
            exact_path,
            descendant_path,
            unrelated_path,
            exact_loader,
            descendant_loader,
            unrelated_loader,
            watcher,
            watched_dirs: HashSet::new(),
            targets,
        };
        fixture.register_dirs();
        fixture
    }

    fn register_dirs(&mut self) {
        for path in [
            &self.exact_path,
            &self.descendant_path,
            &self.unrelated_path,
        ] {
            register_directory(
                &mut self.watcher,
                &mut self.watched_dirs,
                path.parent().expect("fixture parent"),
            );
        }
        add_watched_path(&self.exact_loader, &self.exact_path);
        add_watched_path(&self.descendant_loader, &self.descendant_path);
        add_watched_path(&self.unrelated_loader, &self.unrelated_path);
    }
}

#[test]
fn removed_directory_scope_preserves_unrelated_targets_and_file_removal() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let mut fixture = Fixture::new();
    let exact = fixture
        .exact_path
        .parent()
        .expect("exact parent")
        .to_path_buf();
    let unrelated = fixture
        .unrelated_path
        .parent()
        .expect("unrelated parent")
        .to_path_buf();
    DirectoryRecovery::recover(
        &Event::new(EventKind::Remove(notify::event::RemoveKind::Folder)).add_path(exact.clone()),
        &mut fixture.watcher,
        &mut fixture.watched_dirs,
        &fixture.targets,
    );
    fixture.exact_loader.poll(0);
    fixture.descendant_loader.poll(0);
    fixture.unrelated_loader.poll(0);
    assert!(!fixture.watched_dirs.contains(&exact));
    assert!(
        !fixture
            .watched_dirs
            .contains(fixture.descendant_path.parent().expect("descendant parent"))
    );
    assert!(fixture.watched_dirs.contains(&unrelated));
    assert!(
        fixture
            .exact_loader
            .watch_error(&fixture.exact_path)
            .is_some()
    );
    assert!(
        fixture
            .descendant_loader
            .watch_error(&fixture.descendant_path)
            .is_some()
    );
    assert!(
        !fixture
            .exact_loader
            .inner
            .watched_paths
            .lock()
            .expect("exact watched paths lock")
            .contains(&fixture.exact_path)
    );
    assert!(
        !fixture
            .descendant_loader
            .inner
            .watched_paths
            .lock()
            .expect("descendant watched paths lock")
            .contains(&fixture.descendant_path)
    );
    assert!(
        fixture
            .unrelated_loader
            .watch_error(&fixture.unrelated_path)
            .is_none()
    );
    assert!(fixture.targets.contains_key(&fixture.unrelated_path));

    let mut file_only_dirs = HashSet::from([unrelated]);
    DirectoryRecovery::recover(
        &Event::new(EventKind::Remove(notify::event::RemoveKind::File))
            .add_path(fixture.unrelated_path.clone()),
        &mut fixture.watcher,
        &mut file_only_dirs,
        &fixture.targets,
    );
    assert!(file_only_dirs.contains(fixture.unrelated_path.parent().expect("unrelated parent")));
    assert!(
        fixture
            .unrelated_loader
            .watch_error(&fixture.unrelated_path)
            .is_none()
    );

    notify_error(&mut fixture.targets, "genuine backend failure".to_owned());
    fixture.unrelated_loader.poll(0);
    assert_eq!(
        fixture
            .unrelated_loader
            .watch_error(&fixture.unrelated_path)
            .as_deref(),
        Some("genuine backend failure")
    );
}
