use super::{ContentWatch, Ordering, PathBuf, Targets, WatchEvent, send_event};
use notify::Watcher;

impl ContentWatch {
    pub(super) fn drain_failures(&mut self, targets: &Targets) {
        if self.failure_overflow.swap(false, Ordering::AcqRel) {
            self.handle_failure(
                notify::Error::generic("image content failure queue overflow"),
                targets,
            );
        }
        while let Ok(error) = self.failures.try_recv() {
            self.handle_failure(error, targets);
        }
    }

    fn handle_failure(&mut self, error: notify::Error, targets: &Targets) {
        let paths = if error.paths.is_empty() {
            self.watched.iter().cloned().collect()
        } else {
            error.paths.clone()
        };
        for path in paths {
            if self.watched.contains(&path) {
                match self.watcher.unwatch(&path) {
                    Ok(()) => {
                        self.watched.remove(&path);
                    }
                    Err(failure) if matches!(failure.kind, notify::ErrorKind::WatchNotFound) => {
                        self.watched.remove(&path);
                    }
                    Err(failure) => {
                        tracing::warn!(path = %path.display(), %failure, "failed image content unwatch");
                    }
                }
                self.fail_path(path, &error, targets);
            }
        }
    }

    pub(super) fn fail_path(&mut self, path: PathBuf, error: &notify::Error, targets: &Targets) {
        self.failed.insert(path.clone());
        if let Some(owners) = targets.get(&path) {
            for target in owners {
                if let Some(owner) = target.owner.upgrade() {
                    send_event(
                        &owner,
                        WatchEvent::Failed(
                            target.requested_path.clone(),
                            format!("image content watch failed: {error}"),
                            target.generation,
                        ),
                    );
                }
            }
        }
    }
}
