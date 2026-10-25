use super::{Cache, Inner, LocalImageLoader};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, atomic::AtomicU64, mpsc};

impl Default for LocalImageLoader {
    fn default() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        let (invalidation_tx, invalidation_rx) =
            mpsc::sync_channel(super::INVALIDATION_QUEUE_CAPACITY);
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
