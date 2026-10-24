use notify::{Config, Event, Watcher};

#[cfg(windows)]
use notify::PollWatcher;
#[cfg(not(windows))]
use notify::RecommendedWatcher;
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
pub(super) type ImageWatcher = PollWatcher;
#[cfg(not(windows))]
pub(super) type ImageWatcher = RecommendedWatcher;

#[cfg(windows)]
const WINDOWS_IMAGE_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub(super) struct Backend;

impl Backend {
    pub(super) fn create<F>(handler: F) -> notify::Result<ImageWatcher>
    where
        F: FnMut(notify::Result<Event>) + Send + 'static,
    {
        #[cfg(windows)]
        {
            /* WHY: Windowsの監視handleがdirectory renameを拒否するため、保持handleのないpollingを使う。 */
            ImageWatcher::new(
                handler,
                Config::default().with_poll_interval(WINDOWS_IMAGE_POLL_INTERVAL),
            )
        }
        #[cfg(not(windows))]
        {
            ImageWatcher::new(handler, Config::default())
        }
    }
}
