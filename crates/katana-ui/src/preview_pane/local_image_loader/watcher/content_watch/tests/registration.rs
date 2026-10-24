use super::*;

#[test]
fn registration_acknowledges_after_content_baseline_and_rolls_back_missing_image() {
    let root = tempfile::tempdir().expect("image directory");
    let root_path = std::fs::canonicalize(root.path()).expect("canonical image directory");
    let path = root_path.join("image.png");
    fixed_png(&path, RED);
    let loader = LocalImageLoader::default();
    let (mut content, _) = content();
    let mut parent = Backend::create(|_| {}).expect("parent watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    let request = |path| WatchRequest {
        owner: Arc::downgrade(&loader.inner),
        path,
        generation: loader.generation(),
    };
    Registration::register_with_content(
        request(path.clone()),
        &mut parent,
        &mut directories,
        &mut targets,
        &mut content,
    );
    assert_eq!(content.watched, HashSet::from([path.clone()]));
    let receiver = loader.inner.invalidation_rx.lock().expect("owner events");
    assert!(
        receiver
            .try_iter()
            .any(|event| matches!(event, WatchEvent::Registered(_, _)))
    );
    let missing = root_path.join("missing.png");
    Registration::register_with_content(
        request(missing.clone()),
        &mut parent,
        &mut directories,
        &mut targets,
        &mut content,
    );
    let events = receiver.try_iter().collect::<Vec<_>>();
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, WatchEvent::Registered(value, _) if value == &missing))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WatchEvent::Failed(value, _, _) if value == &missing))
    );
    assert!(!targets.contains_key(&missing));
    assert_eq!(content.watched, HashSet::from([path]));
    assert!(content.failed.is_empty());
}

#[test]
fn missing_content_registration_retries_without_polling_unrelated_files() {
    let root = tempfile::tempdir().expect("image directory");
    let root_path = std::fs::canonicalize(root.path()).expect("canonical image directory");
    let path = root_path.join("missing.png");
    let unrelated = root_path.join("large-office.docx");
    std::fs::write(&unrelated, b"unrelated Office content").expect("unrelated file");
    let loader = LocalImageLoader::default();
    let targets = targets(&loader, &path);
    let directories = HashSet::from([root_path]);
    let (mut watcher, events) = content();
    watcher.sync(&directories, &targets);
    assert!(watcher.watched.is_empty());
    assert_eq!(watcher.failed, HashSet::from([path.clone()]));
    fixed_png(&path, RED);
    watcher.retry(&path);
    watcher.sync(&directories, &targets);
    assert_eq!(watcher.watched, HashSet::from([path.clone()]));
    assert!(watcher.failed.is_empty());
    fixed_png(&path, BLUE);
    let event = changed(&events, &path);
    assert!(!event.paths.contains(&unrelated));
}

#[cfg(unix)]
#[test]
fn alias_and_resolved_content_paths_are_both_registered_and_released() {
    let root = tempfile::tempdir().expect("image directory");
    let directory = std::fs::canonicalize(root.path()).unwrap();
    let resolved = directory.join("real.png");
    let alias = directory.join("alias.png");
    fixed_png(&resolved, RED);
    std::os::unix::fs::symlink(&resolved, &alias).expect("image symlink");
    let loader = LocalImageLoader::default();
    let (mut content, _) = content();
    let mut parent = Backend::create(|_| {}).expect("parent watcher");
    let mut directories = HashSet::new();
    let mut targets = Targets::new();
    Registration::register_with_content(
        WatchRequest {
            owner: Arc::downgrade(&loader.inner),
            path: alias.clone(),
            generation: loader.generation(),
        },
        &mut parent,
        &mut directories,
        &mut targets,
        &mut content,
    );
    assert_eq!(content.watched, HashSet::from([alias, resolved]));
    clear_owner(
        WatchCommand::Clear(Arc::downgrade(&loader.inner)),
        &mut parent,
        &mut directories,
        &mut targets,
    );
    content.sync(&directories, &targets);
    assert!(content.watched.is_empty());
}
