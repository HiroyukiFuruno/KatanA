use super::watcher::WatchEvent;
use super::{LocalImageLoader, watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl LocalImageLoader {
    pub(crate) fn reset(&self) {
        self.inner.generation.fetch_add(1, Ordering::AcqRel);
        self.inner
            .invalidation_overflow
            .store(false, Ordering::Release);
        watcher::coordinator().clear(Arc::downgrade(&self.inner));
        let Ok(mut pending) = self.inner.pending.lock() else {
            return;
        };
        pending.clear();
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.ready.clear();
        cache.textures.clear();
        cache.active_images.clear();
        cache.active_textures.clear();
        cache.bytes = 0;
        let Ok(mut paths) = self.inner.watched_paths.lock() else {
            return;
        };
        paths.clear();
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        pending.clear();
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        errors.clear();
        let Ok(mut context) = self.inner.repaint_context.lock() else {
            return;
        };
        *context = None;
        let Ok(mut revisions) = self.inner.path_revisions.lock() else {
            return;
        };
        revisions.clear();
    }

    pub(crate) fn path_revision(&self, path: &Path) -> u64 {
        self.inner
            .path_revisions
            .lock()
            .ok()
            .and_then(|revisions| revisions.get(path).copied())
            .unwrap_or(0)
    }

    pub(super) fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }

    pub(super) fn watch_error(&self, path: &Path) -> Option<String> {
        self.inner
            .watch_errors
            .lock()
            .ok()
            .and_then(|errors| errors.get(path).cloned())
    }

    pub(super) fn ensure_watched(&self, path: &Path) -> Result<bool, String> {
        let paths = self
            .inner
            .watched_paths
            .lock()
            .map_err(|_| "image watcher unavailable".to_owned())?;
        if paths.contains(path) {
            return Ok(true);
        }
        drop(paths);
        let mut pending = self
            .inner
            .watch_pending
            .lock()
            .map_err(|_| "image watcher unavailable".to_owned())?;
        if pending.contains(path) {
            return Ok(false);
        }
        pending.insert(path.to_path_buf());
        drop(pending);
        watcher::coordinator()
            .request(
                Arc::downgrade(&self.inner),
                path.to_path_buf(),
                self.generation(),
            )
            .inspect_err(|_| self.clear_watch_pending(path))
            .map(|_| false)
    }

    fn clear_watch_pending(&self, path: &Path) {
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        pending.remove(path);
    }

    pub(super) fn poll_invalidations(&self) -> bool {
        let Ok(rx) = self.inner.invalidation_rx.lock() else {
            return false;
        };
        let mut changed = self
            .inner
            .invalidation_overflow
            .swap(false, Ordering::AcqRel);
        if changed {
            self.invalidate_overflowed_paths();
        }
        while let Ok(event) = rx.try_recv() {
            match event {
                WatchEvent::Registered(path, event_generation) => {
                    if event_generation == self.generation() {
                        self.register_watched(path);
                    }
                    changed = true;
                }
                WatchEvent::Changed(path, event_generation) => {
                    if event_generation != self.generation() {
                        continue;
                    }
                    self.invalidate_path(&path);
                    changed = true;
                }
                WatchEvent::Failed(path, error, event_generation) => {
                    if event_generation != self.generation() {
                        continue;
                    }
                    self.invalidate_path(&path);
                    self.store_watch_error(path, error);
                    changed = true;
                }
            }
        }
        changed
    }

    fn register_watched(&self, path: PathBuf) {
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        pending.remove(&path);
        let Ok(mut watched) = self.inner.watched_paths.lock() else {
            return;
        };
        watched.insert(path.clone());
        drop(watched);
        drop(pending);
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        errors.remove(&path);
    }

    fn store_watch_error(&self, path: PathBuf, error: String) {
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        errors.insert(path, error);
    }

    fn invalidate_overflowed_paths(&self) {
        let paths = self
            .inner
            .watched_paths
            .lock()
            .map(|paths| paths.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for path in paths {
            self.invalidate_path(&path);
            self.store_watch_error(path, "image watcher event queue overflowed".to_owned());
        }
    }
}
