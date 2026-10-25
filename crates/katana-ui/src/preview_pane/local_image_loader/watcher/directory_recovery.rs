use super::ImageWatcher;
use super::WatchEvent;
use super::events::{notify_error, notify_event, notify_overflow, send_event};
use super::registration::Targets;
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) struct DirectoryRecovery;

impl DirectoryRecovery {
    pub(super) fn process_event(
        result: notify::Result<Event>,
        watcher: &mut ImageWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        targets: &mut Targets,
    ) {
        match Self::normalize_loss(result, watched_dirs) {
            Ok(event) => {
                Self::recover(&event, watcher, watched_dirs, targets);
                notify_event(event, targets);
            }
            Err(error) if Self::is_anonymous_not_found(&error) => {
                tracing::warn!(
                    ?error,
                    "anonymous image watcher loss could not identify a missing watched directory"
                );
                notify_overflow(targets);
            }
            Err(error) => notify_error(targets, error.to_string()),
        }
    }

    pub(super) fn normalize_loss(
        event: notify::Result<Event>,
        watched_dirs: &HashSet<PathBuf>,
    ) -> notify::Result<Event> {
        let Err(error) = event else {
            return event;
        };
        if !Self::is_anonymous_not_found(&error) {
            return Err(error);
        }

        let paths = Self::missing_watched_dirs(watched_dirs);
        if paths.is_empty() {
            return Err(error);
        }
        Ok(Event {
            kind: EventKind::Remove(notify::event::RemoveKind::Any),
            paths,
            attrs: Default::default(),
        })
    }

    fn is_anonymous_not_found(error: &notify::Error) -> bool {
        error.paths.is_empty()
            && matches!(
                &error.kind,
                notify::ErrorKind::Io(io) if io.kind() == std::io::ErrorKind::NotFound
            )
    }

    fn missing_watched_dirs(watched_dirs: &HashSet<PathBuf>) -> Vec<PathBuf> {
        watched_dirs
            .iter()
            .filter(|directory| {
                matches!(
                    directory.metadata(),
                    Err(ref metadata_error)
                        if metadata_error.kind() == std::io::ErrorKind::NotFound
                )
            })
            .cloned()
            .collect()
    }

    pub(super) fn recover(
        event: &Event,
        watcher: &mut ImageWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        targets: &Targets,
    ) {
        let removed = Self::lost_directories(event, watched_dirs);
        if removed.is_empty() {
            return;
        }
        let backend_changed = Self::detach(watcher, watched_dirs, &removed);
        Self::notify_removed(targets, &removed);
        if backend_changed {
            /* WHY: FSEventsの実解除が他監視のstreamを再開した場合だけ、元の通知欠落補完を維持する。 */
            notify_overflow(targets);
        }
    }

    fn lost_directories(event: &Event, watched_dirs: &HashSet<PathBuf>) -> HashSet<PathBuf> {
        let paths = Self::lost_paths(event);
        if paths.is_empty() {
            return HashSet::new();
        }
        watched_dirs
            .iter()
            .filter(|directory| paths.iter().any(|path| directory.starts_with(path)))
            .cloned()
            .collect()
    }

    fn lost_paths(event: &Event) -> &[PathBuf] {
        match event.kind {
            EventKind::Remove(_)
            | EventKind::Modify(ModifyKind::Name(RenameMode::From | RenameMode::Any)) => {
                &event.paths
            }
            /* WHY: Bothの二番目は移動先なので、移動元とその子だけを監視再登録へ戻す。 */
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                &event.paths[..event.paths.len().min(1)]
            }
            _ => &[],
        }
    }

    fn detach(
        watcher: &mut ImageWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        removed: &HashSet<PathBuf>,
    ) -> bool {
        let mut backend_changed = false;
        for directory in removed {
            /* WHY: inotifyが削除済み監視を自動解除していても、登録済みの台帳を残すと再作成後に再登録できない。 */
            match watcher.unwatch(directory) {
                Ok(()) => backend_changed = true,
                Err(error) if matches!(error.kind, notify::ErrorKind::WatchNotFound) => {}
                Err(error) => {
                    tracing::warn!(path = %directory.display(), %error, "removed image directory unwatch failed");
                }
            }
            watched_dirs.remove(directory);
        }
        backend_changed
    }

    fn notify_removed(targets: &Targets, removed: &HashSet<PathBuf>) {
        for (path, owners) in targets {
            let Some(directory) = path.parent().filter(|parent| removed.contains(*parent)) else {
                continue;
            };
            for target in owners {
                let Some(owner) = target.owner.upgrade() else {
                    continue;
                };
                /* WHY: Changedだけではloaderの登録済み状態が残るため、該当先だけ既存のbounded再登録へ戻す。 */
                send_event(
                    &owner,
                    WatchEvent::Failed(
                        target.requested_path.clone(),
                        format!("image watch directory removed: {}", directory.display()),
                        target.generation,
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "directory_recovery_error_tests.rs"]
mod directory_recovery_error_tests;
