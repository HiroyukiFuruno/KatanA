use super::state::DirectoryMetadata;
use notify::EventHandler;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

pub(super) enum DirectoryCommand {
    Watch(PathBuf, Sender<notify::Result<()>>),
    Unwatch(PathBuf, Sender<notify::Result<()>>),
    Stop,
}

pub(super) struct DirectoryWatchLoop<F> {
    directories: HashSet<PathBuf>,
    handler: F,
}

impl<F: EventHandler> DirectoryWatchLoop<F> {
    pub(super) fn new(handler: F) -> Self {
        Self {
            directories: HashSet::new(),
            handler,
        }
    }

    pub(super) fn run(mut self, commands: Receiver<DirectoryCommand>, interval: Duration) {
        let mut deadline = Instant::now() + interval;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if !self.process(commands.recv_timeout(remaining)) {
                return;
            }
            if Instant::now() >= deadline {
                self.sample();
                deadline = Instant::now() + interval;
            }
        }
    }

    fn process(&mut self, command: Result<DirectoryCommand, RecvTimeoutError>) -> bool {
        match command {
            Ok(DirectoryCommand::Stop) | Err(RecvTimeoutError::Disconnected) => false,
            Ok(DirectoryCommand::Watch(path, reply)) => {
                let _ = reply.send(self.register(path));
                true
            }
            Ok(DirectoryCommand::Unwatch(path, reply)) => {
                let result = self
                    .directories
                    .remove(&path)
                    .then_some(())
                    .ok_or_else(|| notify::Error::watch_not_found().add_path(path));
                let _ = reply.send(result);
                true
            }
            Err(RecvTimeoutError::Timeout) => true,
        }
    }

    fn register(&mut self, path: PathBuf) -> notify::Result<()> {
        DirectoryMetadata::validate(&path)?;
        self.directories.insert(path);
        Ok(())
    }

    fn sample(&mut self) {
        /* WHY: 登録と解除も同じthreadで処理し、解除完了後の古いsample通知を防ぐ。 */
        for path in &self.directories {
            if let Some(event) = DirectoryMetadata::sample(path) {
                self.handler.handle_event(event);
            }
        }
    }
}
