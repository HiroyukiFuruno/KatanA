use super::Inner;
use notify::{Config, RecommendedWatcher, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    Arc, OnceLock, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

mod events;
mod registration;

use registration::{Registration, Targets};

use events::{
    notify_error, notify_event, notify_overflow, retain_live_targets, send_event,
    unwatch_unused_dirs,
};

const WATCH_REQUEST_CAPACITY: usize = 64;

pub(super) enum WatchEvent {
    Registered(PathBuf, u64),
    Changed(PathBuf, u64),
    Overflow(PathBuf, u64),
    Failed(PathBuf, String, u64),
}

struct WatchRequest {
    owner: Weak<Inner>,
    path: PathBuf,
    generation: u64,
}

pub(super) struct Coordinator {
    request_tx: mpsc::SyncSender<WatchCommand>,
    startup_error: Option<String>,
}

enum WatchCommand {
    Watch(WatchRequest),
    Clear(Weak<Inner>),
}

impl Coordinator {
    pub(super) fn request(
        &self,
        owner: Weak<Inner>,
        path: PathBuf,
        generation: u64,
    ) -> Result<(), String> {
        if let Some(error) = &self.startup_error {
            return Err(error.clone());
        }
        self.request_tx
            .try_send(WatchCommand::Watch(WatchRequest {
                owner,
                path,
                generation,
            }))
            .map_err(|error| format!("image watcher queue unavailable: {error}"))
    }

    pub(super) fn clear(&self, owner: Weak<Inner>) {
        if let Err(error) = self.request_tx.try_send(WatchCommand::Clear(owner)) {
            tracing::warn!(%error, "local image watcher clear request dropped");
        }
    }
}

pub(super) fn coordinator() -> &'static Coordinator {
    static INSTANCE: OnceLock<Coordinator> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let (request_tx, request_rx) = mpsc::sync_channel(WATCH_REQUEST_CAPACITY);
        let startup_error = std::thread::Builder::new()
            .name("katana-local-image-watcher".into())
            .spawn(move || run(request_rx));
        Coordinator {
            request_tx,
            startup_error: startup_error.err().map(|error| error.to_string()),
        }
    })
}

fn run(request_rx: mpsc::Receiver<WatchCommand>) {
    let (event_tx, event_rx) = mpsc::sync_channel(256);
    let event_overflow = Arc::new(AtomicBool::new(false));
    let callback_overflow = Arc::clone(&event_overflow);
    let watcher = RecommendedWatcher::new(
        move |event| {
            if event_tx.try_send(event).is_err() {
                callback_overflow.store(true, Ordering::Release);
            }
        },
        Config::default(),
    );
    let mut watcher = match watcher {
        Ok(watcher) => watcher,
        Err(error) => {
            run_without_watcher(request_rx, error.to_string());
            return;
        }
    };
    let mut watched_dirs = HashSet::new();
    let mut targets = Targets::new();
    loop {
        drain_requests(&request_rx, &mut watcher, &mut watched_dirs, &mut targets);
        if event_overflow.swap(false, Ordering::AcqRel) {
            notify_overflow(&targets);
        }
        match event_rx.recv_timeout(Duration::from_millis(25)) {
            Ok(Ok(event)) => notify_event(event, &mut targets),
            Ok(Err(error)) => notify_error(&mut targets, error.to_string()),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
        retain_live_targets(&mut watcher, &mut watched_dirs, &mut targets);
    }
}

fn run_without_watcher(request_rx: mpsc::Receiver<WatchCommand>, error: String) {
    while let Ok(command) = request_rx.recv() {
        if let WatchCommand::Watch(request) = command
            && let Some(owner) = request.owner.upgrade()
        {
            send_event(
                &owner,
                WatchEvent::Failed(request.path, error.clone(), request.generation),
            );
        }
    }
}

fn drain_requests(
    request_rx: &mpsc::Receiver<WatchCommand>,
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &mut Targets,
) {
    while let Ok(command) = request_rx.try_recv() {
        let WatchCommand::Watch(request) = command else {
            clear_owner(command, watcher, watched_dirs, targets);
            continue;
        };
        Registration::register(request, watcher, watched_dirs, targets);
    }
}

fn clear_owner(
    command: WatchCommand,
    watcher: &mut RecommendedWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &mut Targets,
) {
    let WatchCommand::Clear(owner) = command else {
        return;
    };
    targets.retain(|_, owners| {
        owners.retain(|candidate| !candidate.owner.ptr_eq(&owner));
        !owners.is_empty()
    });
    unwatch_unused_dirs(watcher, watched_dirs, targets);
}

#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
