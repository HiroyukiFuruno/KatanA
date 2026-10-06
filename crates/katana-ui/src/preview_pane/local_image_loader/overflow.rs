use super::LocalImageLoader;

impl LocalImageLoader {
    pub(super) fn invalidate_overflowed_paths(&self) {
        let paths = self
            .inner
            .watched_paths
            .lock()
            .map(|paths| paths.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut paths = paths;
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        paths.extend(pending.drain());
        drop(pending);
        for path in paths {
            self.invalidate_path(&path);
        }
    }

    pub(super) fn apply_deferred_watch_results(&self) -> bool {
        let results = self
            .inner
            .deferred_watch_results
            .lock()
            .map(|mut errors| std::mem::take(&mut *errors))
            .unwrap_or_default();
        let mut changed = false;
        for (path, (generation, result)) in results {
            if self.generation() != generation {
                continue;
            }
            match result {
                Ok(()) => self.register_watched(path),
                Err(error) => {
                    self.invalidate_path(&path);
                    self.store_watch_error(path, error, generation);
                }
            }
            changed = true;
        }
        changed
    }
}
