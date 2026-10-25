use super::{Cache, LocalImageLoader, RequestKey, egui, retry};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) type CachedResult = Result<Arc<egui::ColorImage>, DecodeFailure>;

pub(super) fn result_bytes(image: &CachedResult) -> usize {
    image.as_ref().map_or(0, |image| {
        image.pixels.len() * std::mem::size_of::<egui::Color32>()
    })
}

#[derive(Clone)]
pub(super) struct DecodeFailure {
    pub(super) error: String,
    pub(super) attempts: u32,
    pub(super) deadline: Instant,
}

fn old_attempts(cache: &Cache, key: &RequestKey) -> Option<u32> {
    cache
        .ready
        .get(key)
        .and_then(|result| result.as_ref().err())
        .or_else(|| {
            cache
                .active_images
                .get(key)
                .and_then(|(result, _)| result.as_ref().err())
        })
        .map(|failure| failure.attempts)
}

pub(super) fn failure(cache: &Cache, key: &RequestKey, error: String) -> DecodeFailure {
    let attempts = old_attempts(cache, key).map_or(0, |attempts| attempts.saturating_add(1));
    DecodeFailure {
        error,
        attempts,
        deadline: Instant::now() + retry::retry_delay(attempts),
    }
}

fn current_failure<'a>(cache: &'a Cache, key: &RequestKey) -> Option<&'a DecodeFailure> {
    cache
        .ready
        .get(key)
        .and_then(|result| result.as_ref().err())
        .or_else(|| {
            cache
                .active_images
                .get(key)
                .and_then(|(result, _)| result.as_ref().err())
        })
}

pub(super) fn evict_if_due(cache: &mut Cache, key: &RequestKey) -> bool {
    let Some(failure) = current_failure(cache, key) else {
        return false;
    };
    if failure.deadline > Instant::now() {
        return false;
    }
    if cache.ready.contains_key(key) {
        cache.textures.remove(key);
    }
    true
}

pub(super) fn rearm(cache: &mut Cache, key: &RequestKey) {
    let Some(failure) = current_failure(cache, key) else {
        return;
    };
    let deadline = Instant::now() + retry::retry_delay(failure.attempts);
    if let Some(failure) = cache
        .ready
        .get_mut(key)
        .and_then(|result| result.as_mut().err())
    {
        failure.deadline = deadline;
    }
    if let Some((Err(failure), _)) = cache.active_images.get_mut(key) {
        failure.deadline = deadline;
    }
}

impl LocalImageLoader {
    pub(super) fn rearm_decode_failure(&self, key: &RequestKey) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        rearm(&mut cache, key);
    }

    pub(super) fn request_decode_repaint(&self, key: &RequestKey) {
        let deadline = self
            .inner
            .cache
            .lock()
            .ok()
            .and_then(|cache| current_failure(&cache, key).map(|failure| failure.deadline));
        let Some(deadline) = deadline else { return };
        let delay = deadline.saturating_duration_since(Instant::now());
        if let Some(context) = self
            .inner
            .repaint_context
            .lock()
            .ok()
            .and_then(|context| context.clone())
        {
            context.request_repaint_after(delay.min(Duration::from_secs(2)));
        }
    }
}
