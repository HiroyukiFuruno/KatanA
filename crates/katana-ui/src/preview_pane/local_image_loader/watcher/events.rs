use super::WatchEvent;
use super::registration::{Registration, Targets};
use notify::{EventKind, RecommendedWatcher, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

mod queue;

pub(super) use queue::send_event;

pub(super) fn notify_overflow(targets: &Targets) {
    for owners in targets.values() {
        for target in owners {
            if let Some(owner) = target.owner.upgrade() {
                send_event(
                    owner.as_ref(),
                    WatchEvent::Overflow(target.requested_path.clone(), target.generation),
                );
            }
        }
    }
}

pub(super) fn notify_event(event: notify::Event, targets: &mut Targets) {
    if event.need_rescan() {
        notify_overflow(targets);
        return;
    }
    notify_targets(event.kind, &event.paths, targets);
}

pub(super) fn notify_targets(kind: EventKind, paths: &[PathBuf], targets: &mut Targets) {
    if !matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return;
    }
    for path in paths {
        notify_path(path, matches!(kind, EventKind::Remove(_)), targets);
    }
}

fn notify_path(path: &Path, removed: bool, targets: &mut Targets) {
    let paths = match Registration::paths(path) {
        Ok(paths) => paths,
        Err(error) => {
            notify_error(targets, error);
            return;
        }
    };
    for path in paths {
        let matching = targets.iter().filter(|(target_path, _)| {
            **target_path == path || (removed && target_path.starts_with(&path))
        });
        for target in matching.flat_map(|(_, owners)| owners) {
            let Some(owner) = target.owner.upgrade() else {
                continue;
            };
            send_event(
                &owner,
                WatchEvent::Changed(target.requested_path.clone(), target.generation),
            );
        }
    }
}

pub(super) fn notify_error(targets: &mut Targets, error: String) {
    for owners in targets.values() {
        for target in owners {
            if let Some(owner) = target.owner.upgrade() {
                send_event(
                    owner.as_ref(),
                    WatchEvent::Failed(
                        target.requested_path.clone(),
                        error.clone(),
                        target.generation,
                    ),
                );
            }
        }
    }
}

pub(super) fn retain_live_targets(
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &mut Targets,
) {
    targets.retain(|_, owners| {
        owners.retain(|target| {
            target
                .owner
                .upgrade()
                .is_some_and(|owner| owner.generation.load(Ordering::Acquire) == target.generation)
        });
        !owners.is_empty()
    });
    unwatch_unused_dirs(watcher, watched_dirs, targets);
}

pub(super) fn unwatch_unused_dirs(
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &Targets,
) {
    let live_dirs: HashSet<PathBuf> = targets
        .keys()
        .map(|path| {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        })
        .collect();
    let mut changed = false;
    for directory in watched_dirs
        .difference(&live_dirs)
        .cloned()
        .collect::<Vec<_>>()
    {
        if unwatch_directory(watcher, &directory) {
            changed |= watched_dirs.remove(&directory);
        }
    }
    if changed {
        notify_overflow(targets);
    }
}

fn unwatch_directory(watcher: &mut RecommendedWatcher, directory: &Path) -> bool {
    let result = watcher.unwatch(directory);
    let Err(error) = result else {
        return true;
    };
    tracing::warn!(path = %directory.display(), %error, "local image watcher unwatch failed");
    false
}
