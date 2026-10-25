use super::*;
use crate::preview_pane::local_image_loader::watcher_tests::RGBA_CHANNELS;
use std::fs::{File, FileTimes};
use std::time::{Instant, SystemTime};

const FIXTURE_EVENT_CAPACITY: usize = 256;
const FIXTURE_MTIME_SECONDS: u64 = 1_600_000_000;
const EVENT_DEADLINE_SECONDS: u64 = 5;

pub(super) const RED: [u8; RGBA_CHANNELS] = [255, 0, 0, 255];
pub(super) const BLUE: [u8; RGBA_CHANNELS] = [0, 0, 255, 255];
pub(super) const WHITE: eframe::egui::Color32 = crate::theme_bridge::WHITE;

pub(super) fn content() -> (ContentWatch, mpsc::Receiver<notify::Result<Event>>) {
    let (tx, rx) = mpsc::sync_channel(FIXTURE_EVENT_CAPACITY);
    let watcher = ContentWatch::new(tx, Arc::new(AtomicBool::new(false))).expect("content watcher");
    (watcher, rx)
}

pub(super) fn targets(loader: &LocalImageLoader, path: &Path) -> Targets {
    let mut targets = Targets::new();
    targets.insert(
        path.to_path_buf(),
        vec![WatchTarget {
            owner: Arc::downgrade(&loader.inner),
            generation: loader.generation(),
            requested_path: path.to_path_buf(),
        }],
    );
    targets
}

pub(super) fn fixed_png(path: &Path, pixel: [u8; RGBA_CHANNELS]) {
    save_png(path, pixel);
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_MTIME_SECONDS);
    File::options()
        .write(true)
        .open(path)
        .expect("PNG handle")
        .set_times(FileTimes::new().set_modified(time))
        .expect("fixed PNG mtime");
}

pub(super) fn changed(rx: &mpsc::Receiver<notify::Result<Event>>, path: &Path) -> Event {
    let deadline = Instant::now() + Duration::from_secs(EVENT_DEADLINE_SECONDS);
    loop {
        let event = rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("content change before deadline")
            .expect("content watcher event");
        if event.paths.iter().any(|candidate| candidate == path) {
            return event;
        }
    }
}
