use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
    mpsc,
};

mod cache;
mod decode;
mod pool;

use pool::{GlobalPool, ResultMessage, Work};

const REQUEST_CAPACITY: usize = 2;
const IMAGE_CACHE_LIMIT: usize = 64 * 1024 * 1024;
const CACHE_ENTRY_LIMIT: usize = 16;

#[derive(Clone, Hash, PartialEq, Eq)]
struct RequestKey {
    path: PathBuf,
    background: egui::Color32,
}

struct Cache {
    ready: HashMap<RequestKey, Result<Arc<egui::ColorImage>, String>>,
    textures: HashMap<RequestKey, egui::TextureHandle>,
    bytes: usize,
}

struct Inner {
    result_rx: Mutex<mpsc::Receiver<ResultMessage>>,
    result_tx: mpsc::Sender<ResultMessage>,
    pending: Mutex<HashSet<RequestKey>>,
    cache: Mutex<Cache>,
    generation: AtomicU64,
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
        Self {
            inner: Arc::new(Inner {
                result_rx: Mutex::new(result_rx),
                result_tx,
                pending: Mutex::new(HashSet::new()),
                cache: Mutex::new(Cache {
                    ready: HashMap::new(),
                    textures: HashMap::new(),
                    bytes: 0,
                }),
                generation: AtomicU64::new(0),
            }),
        }
    }
}

impl LocalImageLoader {
    pub(crate) fn poll(&self) -> bool {
        let Ok(rx) = self.inner.result_rx.lock() else {
            return false;
        };
        let mut changed = false;
        while let Ok(result) = rx.try_recv() {
            if result.generation != self.generation() {
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
        if let Some(status) = self.cached_status(&key) {
            return status;
        }
        if self.is_pending(&key) {
            return LocalImageStatus::Pending;
        }
        self.enqueue(key)
    }

    pub(crate) fn texture(
        &self,
        ctx: &egui::Context,
        path: &Path,
        background: egui::Color32,
        id: usize,
    ) -> LocalTextureStatus {
        let key = RequestKey {
            path: path.to_path_buf(),
            background,
        };
        if let Ok(cache) = self.inner.cache.lock()
            && let Some(texture) = cache.textures.get(&key)
        {
            return LocalTextureStatus::Ready(texture.clone());
        }
        match self.request(path, background) {
            LocalImageStatus::Ready(image) => self.build_texture(ctx, key, image, id),
            LocalImageStatus::Pending => LocalTextureStatus::Pending,
            LocalImageStatus::Failed(error) => LocalTextureStatus::Failed(error),
        }
    }

    pub(crate) fn reset(&self) {
        self.inner.generation.fetch_add(1, Ordering::AcqRel);
        let Ok(mut pending) = self.inner.pending.lock() else {
            return;
        };
        pending.clear();
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.ready.clear();
        cache.textures.clear();
        cache.bytes = 0;
    }

    fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }

    fn build_texture(
        &self,
        ctx: &egui::Context,
        key: RequestKey,
        image: Arc<egui::ColorImage>,
        id: usize,
    ) -> LocalTextureStatus {
        let texture = ctx.load_texture(
            format!("local_image_{id}"),
            (*image).clone(),
            egui::TextureOptions::LINEAR,
        );
        if Self::color_image_bytes(&image) <= IMAGE_CACHE_LIMIT {
            self.store_texture(key, texture.clone());
        }
        LocalTextureStatus::Ready(texture)
    }

    fn color_image_bytes(image: &egui::ColorImage) -> usize {
        image.pixels.len() * std::mem::size_of::<egui::Color32>()
    }

    fn enqueue(&self, key: RequestKey) -> LocalImageStatus {
        let Some(pool) = GlobalPool::get() else {
            return LocalImageStatus::Failed("image loader unavailable".to_owned());
        };
        let generation = self.generation();
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
        };
        if pool.request_tx.try_send(work).is_err() {
            self.remove_pending(&key);
            return LocalImageStatus::Pending;
        }
        LocalImageStatus::Pending
    }
}

#[cfg(test)]
mod tests;
