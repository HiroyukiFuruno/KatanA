use super::super::super::DeferredWatchResults;
use super::super::{Inner, WatchEvent};
use std::path::PathBuf;
use std::sync::{atomic::Ordering, mpsc};

pub(in super::super) fn send_event(owner: &Inner, event: WatchEvent) {
    let generation = event_generation(&event);
    if owner.generation.load(Ordering::Acquire) != generation {
        return;
    }
    match event {
        WatchEvent::Registered(path, generation) => send_registered(owner, path, generation),
        WatchEvent::Failed(path, error, generation) => send_failed(owner, path, error, generation),
        event => send_regular(owner, event, generation),
    }
    if let Ok(context) = owner.repaint_context.lock()
        && let Some(context) = context.as_ref()
    {
        context.request_repaint();
    }
}

fn event_generation(event: &WatchEvent) -> u64 {
    match event {
        WatchEvent::Registered(_, generation)
        | WatchEvent::Changed(_, generation)
        | WatchEvent::Overflow(_, generation)
        | WatchEvent::Failed(_, _, generation) => *generation,
    }
}

fn send_registered(owner: &Inner, path: PathBuf, generation: u64) {
    let Ok(mut results) = owner.deferred_watch_results.lock() else {
        tracing::error!("local image watcher deferred registration state unavailable");
        return;
    };
    if owner.generation.load(Ordering::Acquire) == generation {
        results.remove(&path);
        try_send_registered(owner, &mut results, path, generation);
    }
}

fn send_failed(owner: &Inner, path: PathBuf, error: String, generation: u64) {
    let Ok(mut results) = owner.deferred_watch_results.lock() else {
        tracing::error!("local image watcher deferred registration state unavailable");
        return;
    };
    if owner.generation.load(Ordering::Acquire) != generation {
        return;
    }
    if let Some(result) = results.get_mut(&path) {
        /* WHY: 通知順を保つため、ドロップ済み登録結果と後続通知をまとめてpoll末尾で反映する。 */
        *result = (generation, Err(error));
        mark_overflow(owner, generation);
        return;
    }
    try_send_failed(owner, &mut results, path, error, generation);
}

fn try_send_registered(
    owner: &Inner,
    results: &mut DeferredWatchResults,
    path: PathBuf,
    generation: u64,
) {
    let Err(error) = owner
        .invalidation_tx
        .try_send(WatchEvent::Registered(path, generation))
    else {
        return;
    };
    let event = match error {
        mpsc::TrySendError::Full(event) | mpsc::TrySendError::Disconnected(event) => event,
    };
    if let WatchEvent::Registered(path, generation) = event {
        results.insert(path, (generation, Ok(())));
    }
    mark_overflow(owner, generation);
}

fn try_send_failed(
    owner: &Inner,
    results: &mut DeferredWatchResults,
    path: PathBuf,
    error: String,
    generation: u64,
) {
    match owner
        .invalidation_tx
        .try_send(WatchEvent::Failed(path, error, generation))
    {
        Ok(()) => {}
        Err(error) => {
            let event = match error {
                mpsc::TrySendError::Full(event) | mpsc::TrySendError::Disconnected(event) => event,
            };
            if let WatchEvent::Failed(path, error, generation) = event {
                results.insert(path, (generation, Err(error)));
            }
            mark_overflow(owner, generation);
        }
    }
}

fn send_regular(owner: &Inner, event: WatchEvent, generation: u64) {
    if owner.invalidation_tx.try_send(event).is_err() {
        mark_overflow(owner, generation);
    }
}

fn mark_overflow(owner: &Inner, generation: u64) {
    if owner.generation.load(Ordering::Acquire) == generation {
        owner
            .invalidation_overflow
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
