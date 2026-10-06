use super::LocalImageLoader;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(100);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_BACKOFF_ATTEMPTS: u32 = 4;

pub(super) struct WatchFailure {
    pub(super) error: String,
    pub(super) generation: u64,
    pub(super) attempts: u32,
    pub(super) deadline: Instant,
}

impl WatchFailure {
    pub(super) fn new(error: String, generation: u64) -> Self {
        Self {
            error,
            generation,
            attempts: 0,
            deadline: Instant::now() + INITIAL_RETRY_DELAY,
        }
    }

    fn next_delay(attempts: u32) -> Duration {
        INITIAL_RETRY_DELAY
            .saturating_mul(2u32.saturating_pow(attempts.min(MAX_BACKOFF_ATTEMPTS)))
            .min(MAX_RETRY_DELAY)
    }
}

impl LocalImageLoader {
    pub(super) fn clear_watch_state(&self, path: &std::path::Path) {
        let Ok(mut pending) = self.inner.watch_pending.lock() else {
            return;
        };
        pending.remove(path);
        drop(pending);
        let Ok(mut watched) = self.inner.watched_paths.lock() else {
            return;
        };
        watched.remove(path);
    }

    pub(super) fn schedule_watch_retry(&self, path: PathBuf, generation: u64) {
        if self.generation() != generation {
            return;
        }
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        let Some(failure) = errors.get_mut(&path) else {
            return;
        };
        failure.deadline = Instant::now() + WatchFailure::next_delay(failure.attempts);
    }

    pub(super) fn retry_watch(&self, path: &std::path::Path) {
        let now = Instant::now();
        let Ok(mut errors) = self.inner.watch_errors.lock() else {
            return;
        };
        let Some(failure) = errors.get_mut(path) else {
            return;
        };
        if failure.generation != self.generation() || failure.deadline > now {
            let deadline = failure.deadline;
            drop(errors);
            self.request_retry_repaint(path, deadline);
            return;
        }
        failure.attempts = failure.attempts.saturating_add(1);
        failure.deadline = now + WatchFailure::next_delay(failure.attempts);
        let generation = failure.generation;
        let next_deadline = failure.deadline;
        drop(errors);
        if let Err(error) = self.ensure_watched(path) {
            self.store_watch_error(path.to_path_buf(), error, generation);
        }
        self.request_retry_repaint(path, next_deadline);
    }

    fn request_retry_repaint(&self, path: &std::path::Path, deadline: Instant) {
        if self
            .inner
            .watch_errors
            .lock()
            .ok()
            .is_none_or(|errors| !errors.contains_key(path))
        {
            return;
        }
        let delay = deadline.saturating_duration_since(Instant::now());
        let context = self
            .inner
            .repaint_context
            .lock()
            .ok()
            .and_then(|context| context.clone());
        if let Some(context) = context {
            context.request_repaint_after(delay);
        }
    }
}
