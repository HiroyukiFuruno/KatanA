use super::registration::{Registration, Targets};
use super::{WatchEvent, send_event};
use notify::{Config, Event, PollWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

mod failure;

const CONTENT_FAILURE_CAPACITY: usize = 256;
const CONTENT_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub(super) struct ContentWatch {
    watcher: PollWatcher,
    watched: HashSet<PathBuf>,
    failed: HashSet<PathBuf>,
    failures: mpsc::Receiver<notify::Error>,
    failure_overflow: Arc<AtomicBool>,
}

impl ContentWatch {
    #[cfg(any(windows, test))]
    pub(super) fn register(
        &mut self,
        request: &super::WatchRequest,
        directories: &HashSet<PathBuf>,
        targets: &Targets,
    ) -> Result<(), String> {
        self.retry(&request.path);
        self.sync(directories, targets);
        let paths = Registration::paths(&request.path)?;
        if paths.iter().all(|path| self.watched.contains(path)) {
            /* WHY: baseline確立前に旧画像がdecode済みでも、対象ownerだけを一度再確認する。 */
            if let Some(owner) = request.owner.upgrade() {
                send_event(
                    &owner,
                    WatchEvent::Overflow(request.path.clone(), request.generation),
                );
            }
            Ok(())
        } else {
            Err("image content watch registration failed".into())
        }
    }

    pub(super) fn new(
        events: mpsc::SyncSender<notify::Result<Event>>,
        overflow: Arc<AtomicBool>,
    ) -> notify::Result<Self> {
        let (failure_tx, failures) = mpsc::sync_channel(CONTENT_FAILURE_CAPACITY);
        let failure_overflow = Arc::new(AtomicBool::new(false));
        let callback_failure_overflow = Arc::clone(&failure_overflow);
        let watcher = PollWatcher::new(
            move |event| {
                let full = match event {
                    Ok(event) => events.try_send(Ok(event)).is_err(),
                    Err(error) => {
                        let full = failure_tx.try_send(error).is_err();
                        if full {
                            callback_failure_overflow.store(true, Ordering::Release);
                        }
                        full
                    }
                };
                if full {
                    overflow.store(true, Ordering::Release);
                }
            },
            Config::default()
                .with_poll_interval(CONTENT_POLL_INTERVAL)
                .with_compare_contents(true),
        )?;
        Ok(Self {
            watcher,
            watched: HashSet::new(),
            failed: HashSet::new(),
            failures,
            failure_overflow,
        })
    }

    pub(super) fn retry(&mut self, requested: &Path) {
        let Ok(paths) = Registration::paths(requested) else {
            return;
        };
        for path in paths {
            self.failed.remove(&path);
        }
    }

    pub(super) fn sync(&mut self, directories: &HashSet<PathBuf>, targets: &Targets) {
        self.drain_failures(targets);
        /* WHY: 親全体を内容走査すると巨大Officeも読むため、実際の画像経路だけをhash比較する。 */
        let desired: HashSet<_> = targets
            .keys()
            .filter(|path| {
                path.parent()
                    .is_some_and(|parent| directories.contains(parent))
            })
            .cloned()
            .collect();
        self.failed.retain(|path| desired.contains(path));
        self.remove_unused(&desired);
        for path in desired
            .difference(&self.watched)
            .cloned()
            .collect::<Vec<_>>()
        {
            if !self.failed.contains(&path) {
                self.install(path, targets);
            }
        }
    }

    fn remove_unused(&mut self, desired: &HashSet<PathBuf>) {
        for path in self
            .watched
            .difference(desired)
            .cloned()
            .collect::<Vec<_>>()
        {
            match self.watcher.unwatch(&path) {
                Ok(()) => {
                    self.watched.remove(&path);
                }
                Err(error) if matches!(error.kind, notify::ErrorKind::WatchNotFound) => {
                    self.watched.remove(&path);
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "image content unwatch failed");
                }
            }
        }
    }

    fn install(&mut self, path: PathBuf, targets: &Targets) {
        let result = std::fs::metadata(&path)
            .and_then(|metadata| {
                metadata
                    .is_file()
                    .then_some(())
                    .ok_or_else(|| std::io::Error::other("image content watch requires a file"))
            })
            .map_err(notify::Error::io)
            .and_then(|()| self.watcher.watch(&path, RecursiveMode::NonRecursive));
        match result {
            Ok(()) => {
                self.watched.insert(path);
            }
            Err(error) => {
                self.fail_path(path, &error, targets);
            }
        }
    }
}

#[cfg(test)]
mod tests;
