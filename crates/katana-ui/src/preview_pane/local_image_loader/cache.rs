use super::{
    Arc, CACHE_ENTRY_LIMIT, Cache, IMAGE_CACHE_LIMIT, LocalImageLoader, RequestKey, decode_retry,
    egui,
};
use std::sync::atomic::Ordering;

impl LocalImageLoader {
    pub(super) fn evict_failed_decodes(&self, path: &std::path::Path) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        /* WHY: watch復旧を待たず実decodeを再試行するが、成功画像と別pathのcacheは保持する。失敗entryの使用bytesは0。 */
        cache
            .ready
            .retain(|key, image| key.path != path || image.is_ok());
        cache
            .active_images
            .retain(|key, (image, _)| key.path != path || image.is_ok());
    }

    pub(super) fn advance_active_frame(&self, frame: u64) {
        let previous = self.inner.active_frame.swap(frame, Ordering::AcqRel);
        if previous == frame {
            return;
        }
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        /* WHY: 同じframeの複数描画では破棄せず、前frameに表示された画像にも一回の猶予を与える。 */
        let oldest = frame.saturating_sub(1);
        cache.active_images.retain(|_, (_, seen)| *seen >= oldest);
        cache.active_textures.retain(|_, (_, seen)| *seen >= oldest);
    }

    pub(super) fn build_texture(
        &self,
        ctx: &egui::Context,
        key: RequestKey,
        image: Arc<egui::ColorImage>,
        id: usize,
    ) -> super::LocalTextureStatus {
        let bytes = image.pixels.len() * std::mem::size_of::<egui::Color32>();
        let texture = ctx.load_texture(
            format!("local_image_{id}"),
            image,
            egui::TextureOptions::LINEAR,
        );
        self.trace_texture_queued(&key.path, texture.id());
        self.store_active_texture(key.clone(), texture.clone());
        if bytes <= IMAGE_CACHE_LIMIT {
            self.store_texture(key, texture.clone());
        }
        super::LocalTextureStatus::Ready(texture)
    }

    pub(crate) fn release_active_image(&self, path: &std::path::Path, background: egui::Color32) {
        let key = RequestKey {
            path: path.to_path_buf(),
            background,
        };
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.active_images.remove(&key);
    }

    pub(super) fn is_pending(&self, key: &RequestKey) -> bool {
        let Ok(pending) = self.inner.pending.lock() else {
            return false;
        };
        pending.contains(key)
    }

    pub(super) fn remove_pending(&self, key: &RequestKey) {
        let Ok(mut pending) = self.inner.pending.lock() else {
            return;
        };
        pending.remove(key);
    }

    pub(super) fn cached_status(&self, key: &RequestKey) -> Option<super::LocalImageStatus> {
        let mut cache = self.inner.cache.lock().ok()?;
        let frame = self.inner.active_frame.load(Ordering::Acquire);
        if let Some((_, seen)) = cache.active_images.get_mut(key) {
            *seen = frame;
        }
        if decode_retry::evict_if_due(&mut cache, key) {
            return None;
        }
        if let Some(image) = cache.ready.get(key) {
            return Some(match image {
                Ok(image) => super::LocalImageStatus::Ready(Arc::clone(image)),
                Err(error) => super::LocalImageStatus::Failed(error.error.clone()),
            });
        }
        if let Some((image, seen)) = cache.active_images.get_mut(key) {
            *seen = frame;
            return Some(match image {
                Ok(image) => super::LocalImageStatus::Ready(Arc::clone(image)),
                Err(error) => super::LocalImageStatus::Failed(error.error.clone()),
            });
        }
        None
    }

    pub(super) fn store_result(&self, key: RequestKey, image: Result<egui::ColorImage, String>) {
        let bytes = image.as_ref().map_or(0, |image| {
            image.pixels.len() * std::mem::size_of::<egui::Color32>()
        });
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        let frame = self.inner.active_frame.load(Ordering::Acquire);
        if bytes > IMAGE_CACHE_LIMIT {
            store_oversized(&mut cache, key, image, frame);
            return;
        }
        store_cached(&mut cache, key, image, bytes, frame);
    }

    pub(super) fn store_texture(&self, key: RequestKey, texture: egui::TextureHandle) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.textures.insert(key.clone(), texture);
        trim_cache(&mut cache, &key);
    }

    pub(super) fn store_active_texture(&self, key: RequestKey, texture: egui::TextureHandle) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.active_images.remove(&key);
        let frame = self.inner.active_frame.load(Ordering::Acquire);
        cache.active_textures.insert(key, (texture, frame));
    }
}
fn store_oversized(
    cache: &mut Cache,
    key: RequestKey,
    image: Result<egui::ColorImage, String>,
    frame: u64,
) {
    if let Some(old) = cache.ready.remove(&key) {
        cache.bytes = cache.bytes.saturating_sub(decode_retry::result_bytes(&old));
    }
    cache.textures.remove(&key);
    cache.active_textures.remove(&key);
    let Ok(image) = image else {
        return;
    };
    cache
        .active_images
        .insert(key.clone(), (Ok(Arc::new(image)), frame));
    trim_cache(cache, &key);
}

fn store_cached(
    cache: &mut Cache,
    key: RequestKey,
    image: Result<egui::ColorImage, String>,
    bytes: usize,
    frame: u64,
) {
    cache.bytes = cache
        .bytes
        .saturating_sub(cache.ready.get(&key).map_or(0, decode_retry::result_bytes));
    cache.bytes += bytes;
    let result = image.map_or_else(
        |error| Err(decode_retry::failure(cache, &key, error)),
        |image| Ok(Arc::new(image)),
    );
    cache.ready.insert(key.clone(), result.clone());
    cache.active_images.insert(key.clone(), (result, frame));
    trim_cache(cache, &key);
}

fn trim_cache(cache: &mut Cache, retained: &RequestKey) {
    while (cache.bytes > IMAGE_CACHE_LIMIT || cache.ready.len() > CACHE_ENTRY_LIMIT)
        && cache.ready.len() > 1
    {
        /* WHY: 受信直後の結果を即座に退避すると初描画前に再読込が必要になる。 */
        let Some(key) = cache.ready.keys().find(|key| *key != retained).cloned() else {
            break;
        };
        if let Some(old) = cache.ready.remove(&key) {
            cache.bytes = cache.bytes.saturating_sub(decode_retry::result_bytes(&old));
        }
        cache.textures.remove(&key);
    }
}
