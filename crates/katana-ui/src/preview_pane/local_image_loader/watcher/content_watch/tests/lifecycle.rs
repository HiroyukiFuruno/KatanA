use super::*;

#[test]
fn owner_drop_and_last_target_clear_release_content_watches() {
    let root = tempfile::tempdir().expect("image directory");
    let path = root.path().join("image.png");
    fixed_png(&path, RED);
    let first = LocalImageLoader::default();
    let second = LocalImageLoader::default();
    let mut targets = targets(&first, &path);
    targets.get_mut(&path).unwrap().push(WatchTarget {
        owner: Arc::downgrade(&second.inner),
        generation: second.generation(),
        requested_path: path.clone(),
    });
    let mut directories = HashSet::from([root.path().to_path_buf()]);
    let mut parent = Backend::create(|_| {}).expect("parent watcher");
    parent
        .watch(root.path(), RecursiveMode::NonRecursive)
        .expect("parent watch");
    let (mut watcher, _) = content();
    watcher.sync(&directories, &targets);
    drop(first);
    retain_live_targets(&mut parent, &mut directories, &mut targets);
    watcher.sync(&directories, &targets);
    assert_eq!(watcher.watched, HashSet::from([path.clone()]));
    clear_owner(
        WatchCommand::Clear(Arc::downgrade(&second.inner)),
        &mut parent,
        &mut directories,
        &mut targets,
    );
    watcher.sync(&directories, &targets);
    assert!(watcher.watched.is_empty());
    assert!(watcher.failed.is_empty());
    assert!(targets.is_empty());
}

#[test]
fn detached_parent_releases_content_and_recreated_parent_can_register_again() {
    let root = tempfile::tempdir().expect("fixture directory");
    let directory = root.path().join("images");
    std::fs::create_dir(&directory).expect("images directory");
    let path = directory.join("image.png");
    fixed_png(&path, RED);
    let loader = LocalImageLoader::default();
    let targets = targets(&loader, &path);
    let (mut watcher, events) = content();
    watcher.sync(&HashSet::from([directory.clone()]), &targets);
    let moved = root.path().join("moved");
    std::fs::rename(&directory, &moved).expect("rename content-watched parent");
    watcher.sync(&HashSet::new(), &targets);
    assert!(watcher.watched.is_empty());
    std::fs::rename(&moved, &directory).expect("restore parent");
    watcher.retry(&path);
    watcher.sync(&HashSet::from([directory]), &targets);
    while events.try_recv().is_ok() {}
    fixed_png(&path, BLUE);
    changed(&events, &path);
}

#[test]
fn lost_content_failure_is_retried_instead_of_retaining_a_stale_registration() {
    let root = tempfile::tempdir().expect("image directory");
    let path = std::fs::canonicalize(root.path())
        .unwrap()
        .join("image.png");
    fixed_png(&path, RED);
    let loader = LocalImageLoader::default();
    let targets = targets(&loader, &path);
    let directories = HashSet::from([path.parent().unwrap().to_path_buf()]);
    let (mut watcher, _) = content();
    watcher.sync(&directories, &targets);
    watcher.failure_overflow.store(true, Ordering::Release);
    watcher.sync(&directories, &targets);
    assert!(watcher.watched.is_empty());
    assert_eq!(watcher.failed, HashSet::from([path.clone()]));
    assert!(
        loader
            .inner
            .invalidation_rx
            .lock()
            .unwrap()
            .try_iter()
            .any(|event| matches!(event, WatchEvent::Failed(value, _, _) if value == path))
    );
    watcher.retry(&path);
    watcher.sync(&directories, &targets);
    assert_eq!(watcher.watched, HashSet::from([path]));
    assert!(watcher.failed.is_empty());
}
