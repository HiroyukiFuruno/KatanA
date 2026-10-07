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
        drop(pending);
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.ready.clear();
        cache.textures.clear();
        cache.active_images.clear();
        cache.active_textures.clear();
        cache.bytes = 0;
        drop(cache);
        let Ok(mut paths) = self.inner.watched_paths.lock() else {
            return;
        };
        paths.clear();
        drop(paths);
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        pending.clear();
        drop(pending);
        let Ok(mut deferred_results) = self.inner.deferred_watch_results.lock() else {
            return;
        };
        deferred_results.clear();
        drop(deferred_results);
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        errors.clear();
        drop(errors);
        let Ok(mut context) = self.inner.repaint_context.lock() else {
            return;
        };
        *context = None;
        drop(context);
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
            .and_then(|errors| errors.get(path).map(|failure| failure.error.clone()))
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
            changed |= self.process_watch_event(event);
        }
        changed |= self.apply_deferred_watch_results();
        changed
    }

    fn process_watch_event(&self, event: WatchEvent) -> bool {
        match event {
            WatchEvent::Registered(path, generation) => {
                if generation == self.generation() {
                    self.register_watched(path);
                }
            }
            WatchEvent::Changed(path, generation) | WatchEvent::Overflow(path, generation) => {
                if generation != self.generation() {
                    return false;
                }
                self.invalidate_path(&path);
            }
            WatchEvent::Failed(path, error, generation) => {
                if generation != self.generation() {
                    return false;
                }
                self.clear_watch_state(&path);
                self.store_watch_error(path.clone(), error, generation);
                self.schedule_watch_retry(path, generation);
            }
        }
        true
    }

    pub(super) fn register_watched(&self, path: PathBuf) {
        let had_error = self
            .inner
            .watch_errors
            .lock()
            .ok()
            .is_some_and(|mut errors| errors.remove(&path).is_some());
        if had_error {
            self.invalidate_path(&path);
        }
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
    }

    pub(super) fn store_watch_error(&self, path: PathBuf, error: String, generation: u64) {
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        if self.generation() == generation {
            if let Some(failure) = errors.get_mut(&path) {
                failure.error = error;
                failure.generation = generation;
            } else {
                errors.insert(path, super::retry::WatchFailure::new(error, generation));
            }
        }
    }
}
