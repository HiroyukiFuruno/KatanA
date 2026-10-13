use super::WatchEvent;
use super::events::{notify_overflow, send_event};
use super::registration::Targets;
use notify::{Event, EventKind, RecommendedWatcher, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) struct DirectoryRecovery;

impl DirectoryRecovery {
    pub(super) fn recover(
        event: &Event,
        watcher: &mut RecommendedWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        targets: &Targets,
    ) {
        if !matches!(event.kind, EventKind::Remove(_)) {
            return;
        }
        let removed: HashSet<_> = watched_dirs
            .iter()
            .filter(|directory| event.paths.iter().any(|path| directory.starts_with(path)))
            .cloned()
            .collect();
        let backend_changed = Self::detach(watcher, watched_dirs, &removed);
        Self::notify_removed(targets, &removed);
        if backend_changed {
            /* WHY: FSEventsの実解除が他監視のstreamを再開した場合だけ、元の通知欠落補完を維持する。 */
            notify_overflow(targets);
        }
    }

    fn detach(
        watcher: &mut RecommendedWatcher,
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
