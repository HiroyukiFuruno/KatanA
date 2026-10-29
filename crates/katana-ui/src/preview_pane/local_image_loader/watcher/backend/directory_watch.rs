use notify::{Config, EventHandler, RecursiveMode, Watcher, WatcherKind};
use std::path::Path;
use std::sync::mpsc;
use std::thread::JoinHandle;

#[path = "directory_watch_state.rs"]
mod state;
#[path = "directory_watch_loop.rs"]
mod worker;

use worker::{DirectoryCommand, DirectoryWatchLoop};

pub(in crate::preview_pane::local_image_loader::watcher) struct DirectoryWatcher {
    commands: mpsc::Sender<DirectoryCommand>,
    thread: Option<JoinHandle<()>>,
}

impl DirectoryWatcher {
    fn request(
        &self,
        command: impl FnOnce(mpsc::Sender<notify::Result<()>>) -> DirectoryCommand,
    ) -> notify::Result<()> {
        let (reply, result) = mpsc::channel();
        self.commands.send(command(reply))?;
        result.recv()?
    }
}

impl Watcher for DirectoryWatcher {
    fn new<F: EventHandler>(handler: F, config: Config) -> notify::Result<Self> {
        let interval = config
            .poll_interval()
            .filter(|interval| !interval.is_zero())
            .ok_or_else(|| notify::Error::invalid_config(&config))?;
        if config.compare_contents() {
            return Err(notify::Error::invalid_config(&config));
        }
        let (commands, receiver) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("katana-image-directory-watch".into())
            .spawn(move || DirectoryWatchLoop::new(handler).run(receiver, interval))?;
        Ok(Self {
            commands,
            thread: Some(thread),
        })
    }

    fn watch(&mut self, path: &Path, recursive: RecursiveMode) -> notify::Result<()> {
        if matches!(recursive, RecursiveMode::Recursive) {
            return Err(
                notify::Error::generic("directory metadata watch is nonrecursive")
                    .add_path(path.to_path_buf()),
            );
        }
        self.request(|reply| DirectoryCommand::Watch(path.to_path_buf(), reply))
    }

    fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
        self.request(|reply| DirectoryCommand::Unwatch(path.to_path_buf(), reply))
    }

    fn kind() -> WatcherKind {
        WatcherKind::PollWatcher
    }
}

impl Drop for DirectoryWatcher {
    fn drop(&mut self) {
        /* WHY: 停止要求で待機を起こし、fixtureや所有者の解放前に監視threadを終える。 */
        let _ = self.commands.send(DirectoryCommand::Stop);
        if self
            .thread
            .take()
            .is_some_and(|thread| thread.join().is_err())
        {
            tracing::error!("image directory watcher thread panicked");
        }
    }
}

#[cfg(test)]
#[path = "directory_watch_lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "directory_watch_tests.rs"]
mod tests;
