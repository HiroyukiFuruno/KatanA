use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::AtomicU64, mpsc};

mod cache;
mod decode;
mod overflow;
mod pool;
mod retry;
mod revision;
mod revision_cache;
mod texture;
mod watcher;

use pool::{GlobalPool, ResultMessage, Work};
use watcher::WatchEvent;

const REQUEST_CAPACITY: usize = 2;
const INVALIDATION_QUEUE_CAPACITY: usize = 256;
const IMAGE_CACHE_LIMIT: usize = 64 * 1024 * 1024;
const CACHE_ENTRY_LIMIT: usize = 16;

type DeferredWatchResults = HashMap<PathBuf, (u64, Result<(), String>)>;

#[derive(Clone, Hash, PartialEq, Eq)]
struct RequestKey {
    path: PathBuf,
    background: egui::Color32,
}

struct Cache {
    ready: HashMap<RequestKey, Result<Arc<egui::ColorImage>, String>>,
    textures: HashMap<RequestKey, egui::TextureHandle>,
    active_images: HashMap<RequestKey, (Result<Arc<egui::ColorImage>, String>, u64)>,
    active_textures: HashMap<RequestKey, (egui::TextureHandle, u64)>,
    bytes: usize,
}

struct Inner {
    result_rx: Mutex<mpsc::Receiver<ResultMessage>>,
    result_tx: mpsc::Sender<ResultMessage>,
    invalidation_rx: Mutex<mpsc::Receiver<WatchEvent>>,
    invalidation_tx: mpsc::SyncSender<WatchEvent>,
    invalidation_overflow: std::sync::atomic::AtomicBool,
    repaint_context: Mutex<Option<egui::Context>>,
    pending: Mutex<HashSet<RequestKey>>,
    path_revisions: Mutex<HashMap<PathBuf, u64>>,
    watched_paths: Mutex<HashSet<PathBuf>>,
    watch_pending: Mutex<HashSet<PathBuf>>,
    watch_errors: Mutex<HashMap<PathBuf, retry::WatchFailure>>,
    deferred_watch_results: Mutex<DeferredWatchResults>,
    cache: Mutex<Cache>,
    generation: AtomicU64,
    active_frame: AtomicU64,
}

#[derive(Clone)]
pub(crate) struct LocalImageLoader {
    inner: Arc<Inner>,
}

pub(crate) enum LocalImageStatus {
    Pending,
    Ready(Arc<egui::ColorImage>),
    Failed(String),
}

pub(crate) enum LocalTextureStatus {
    Pending,
    Ready(egui::TextureHandle),
    Failed(String),
}

impl Default for LocalImageLoader {
    fn default() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        let (invalidation_tx, invalidation_rx) = mpsc::sync_channel(INVALIDATION_QUEUE_CAPACITY);
        Self {
            inner: Arc::new(Inner {
                result_rx: Mutex::new(result_rx),
                result_tx,
                invalidation_rx: Mutex::new(invalidation_rx),
                invalidation_tx,
                invalidation_overflow: std::sync::atomic::AtomicBool::new(false),
                repaint_context: Mutex::new(None),
                pending: Mutex::new(HashSet::new()),
                path_revisions: Mutex::new(HashMap::new()),
                watched_paths: Mutex::new(HashSet::new()),
                watch_pending: Mutex::new(HashSet::new()),
                watch_errors: Mutex::new(HashMap::new()),
                deferred_watch_results: Mutex::new(HashMap::new()),
                cache: Mutex::new(Cache {
                    ready: HashMap::new(),
                    textures: HashMap::new(),
                    active_images: HashMap::new(),
                    active_textures: HashMap::new(),
                    bytes: 0,
                }),
                generation: AtomicU64::new(0),
                active_frame: AtomicU64::new(0),
            }),
        }
    }
}

impl LocalImageLoader {
    pub(crate) fn poll(&self, frame: u64) -> bool {
        self.advance_active_frame(frame);
        let mut changed = self.poll_invalidations();
        let Ok(rx) = self.inner.result_rx.lock() else {
            return false;
        };
        while let Ok(result) = rx.try_recv() {
            if result.generation != self.generation()
                || result.revision != self.path_revision(&result.key.path)
            {
                continue;
            }
            changed = true;
            self.remove_pending(&result.key);
            self.store_result(result.key, result.image);
        }
        changed
    }

    pub(crate) fn request(&self, path: &Path, background: egui::Color32) -> LocalImageStatus {
        let key = RequestKey {
            path: path.to_path_buf(),
            background,
        };
        let watch_failed = self.watch_error(&key.path).is_some();
        if watch_failed {
            self.retry_watch(&key.path);
        }
        if let Some(status) = self.cached_status(&key) {
            return status;
        }
        if !watch_failed {
            match self.ensure_watched(&key.path) {
                Ok(true) => {}
                Ok(false) => return LocalImageStatus::Pending,
                Err(error) => {
                    let generation = self.generation();
                    self.store_watch_error(key.path.clone(), error, generation);
                    self.schedule_watch_retry(key.path.clone(), generation);
                }
            }
        }
        if self.is_pending(&key) {
            return LocalImageStatus::Pending;
        }
        self.enqueue(key)
    }

    fn enqueue(&self, key: RequestKey) -> LocalImageStatus {
        let Some(pool) = GlobalPool::get() else {
            return LocalImageStatus::Failed("image loader unavailable".to_owned());
        };
        let generation = self.generation();
        let revision = self.path_revision(&key.path);
        let Ok(mut pending) = self.inner.pending.lock() else {
            return LocalImageStatus::Failed("image loader unavailable".to_owned());
        };
        pending.insert(key.clone());
        drop(pending);
        let work = Work {
            owner: Arc::downgrade(&self.inner),
            result_tx: self.inner.result_tx.clone(),
            key: key.clone(),
            generation,
            revision,
        };
        if pool.request_tx.try_send(work).is_err() {
            self.remove_pending(&key);
            return LocalImageStatus::Failed("image loader queue is full".to_owned());
        }
        LocalImageStatus::Pending
    }
}

#[cfg(test)]
mod failed_decode_retry_tests;
#[cfg(test)]
mod fullscreen_gui_tests;
#[cfg(test)]
mod gui_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod batch_tests;

#[cfg(test)]
mod display_tests;
#[cfg(test)]
mod retry_tests;
#[cfg(test)]
mod watcher_overflow_tests;
#[cfg(test)]
mod watcher_tests;
