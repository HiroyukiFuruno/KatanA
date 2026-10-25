use super::Inner;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    Arc, OnceLock, Weak,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

mod backend;
#[cfg(any(windows, test))]
mod content_watch;
mod directory_recovery;
mod events;
mod registration;

use backend::{Backend, ImageWatcher};

use registration::{Registration, Targets};

use events::{notify_overflow, retain_live_targets, send_event, unwatch_unused_dirs};

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
    #[cfg(windows)]
    let mut content =
        match content_watch::ContentWatch::new(event_tx.clone(), Arc::clone(&event_overflow)) {
            Ok(content) => content,
            Err(error) => return run_without_watcher(request_rx, error.to_string()),
        };
    let watcher = Backend::create(move |event| {
        if event_tx.try_send(event).is_err() {
            callback_overflow.store(true, Ordering::Release);
        }
    });
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
        drain_requests(
            &request_rx,
            &mut watcher,
            &mut watched_dirs,
            &mut targets,
            #[cfg(windows)]
            &mut content,
        );
        #[cfg(windows)]
        content.sync(&watched_dirs, &targets);
        if event_overflow.swap(false, Ordering::AcqRel) {
            notify_overflow(&targets);
        }
        match event_rx.recv_timeout(Duration::from_millis(25)) {
            Ok(event) => directory_recovery::DirectoryRecovery::process_event(
                event,
                &mut watcher,
                &mut watched_dirs,
                &mut targets,
            ),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
        retain_live_targets(&mut watcher, &mut watched_dirs, &mut targets);
        #[cfg(windows)]
        content.sync(&watched_dirs, &targets);
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
    watcher: &mut ImageWatcher,
    watched_dirs: &mut HashSet<PathBuf>,
    targets: &mut Targets,
    #[cfg(windows)] content: &mut content_watch::ContentWatch,
) {
    while let Ok(command) = request_rx.try_recv() {
        let WatchCommand::Watch(request) = command else {
            clear_owner(command, watcher, watched_dirs, targets);
            continue;
        };
        #[cfg(windows)]
        Registration::register_with_content(request, watcher, watched_dirs, targets, content);
        #[cfg(not(windows))]
        Registration::register(request, watcher, watched_dirs, targets);
    }
}

fn clear_owner(
    command: WatchCommand,
    watcher: &mut ImageWatcher,
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
mod directory_lifecycle_tests;
#[cfg(all(test, target_os = "linux"))]
mod directory_recovery_tests;
#[cfg(test)]
mod event_path_tests;
#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod rename_classification_tests;
#[cfg(test)]
mod rename_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
#[cfg(all(test, target_os = "windows"))]
mod windows_rename_diagnostics;
