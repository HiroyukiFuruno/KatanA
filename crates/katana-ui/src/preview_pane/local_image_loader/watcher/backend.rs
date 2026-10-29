use notify::{Config, Event, Watcher};

#[cfg(any(windows, test))]
mod directory_watch;

#[cfg(not(windows))]
use notify::RecommendedWatcher;
#[cfg(any(windows, test))]
use std::time::Duration;

#[cfg(windows)]
pub(super) type ImageWatcher = directory_watch::DirectoryWatcher;
#[cfg(not(windows))]
pub(super) type ImageWatcher = RecommendedWatcher;

#[cfg(any(windows, test))]
const WINDOWS_IMAGE_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub(super) struct Backend;

impl Backend {
    pub(super) fn create<F>(handler: F) -> notify::Result<ImageWatcher>
    where
        F: FnMut(notify::Result<Event>) + Send + 'static,
    {
        #[cfg(windows)]
        {
            let mut handler = handler;
            /* WHY: directory列挙とrenameの競合を避け、存在とtypeだけを監視する。 */
            ImageWatcher::new(
                move |event| handler(Self::normalize_loss(event)),
                Config::default().with_poll_interval(WINDOWS_IMAGE_POLL_INTERVAL),
            )
        }
        #[cfg(not(windows))]
        {
            ImageWatcher::new(handler, Config::default())
        }
    }

    #[cfg(any(windows, test))]
    fn normalize_loss(event: notify::Result<Event>) -> notify::Result<Event> {
        match event {
            Err(error)
                if matches!(&error.kind, notify::ErrorKind::Io(io) if io.kind() == std::io::ErrorKind::NotFound)
                    && !error.paths.is_empty() =>
            {
                /* WHY: pollingの欠損通知も実ディレクトリ消失として既存の解除・復旧へ渡す。 */
                Ok(Event {
                    kind: notify::EventKind::Remove(notify::event::RemoveKind::Any),
                    paths: error.paths,
                    attrs: Default::default(),
                })
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Backend;
    use notify::{Error, EventKind, event::RemoveKind};

    #[test]
    fn missing_polled_root_is_a_real_loss_but_other_errors_remain_errors() {
        let missing = Error::io(std::io::Error::from(std::io::ErrorKind::NotFound))
            .add_path(std::path::PathBuf::from("removed"));
        let event = Backend::normalize_loss(Err(missing)).expect("directory loss");
        assert_eq!(event.kind, EventKind::Remove(RemoveKind::Any));
        assert_eq!(event.paths, [std::path::PathBuf::from("removed")]);
        for kind in [
            std::io::ErrorKind::PermissionDenied,
            std::io::ErrorKind::Other,
        ] {
            let error = Error::io(std::io::Error::from(kind))
                .add_path(std::path::PathBuf::from("still-watched"));
            assert!(Backend::normalize_loss(Err(error)).is_err());
        }
        let no_path = Error::io(std::io::Error::from(std::io::ErrorKind::NotFound));
        assert!(Backend::normalize_loss(Err(no_path)).is_err());
    }
}
