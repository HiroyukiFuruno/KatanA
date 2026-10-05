use super::{Arc, CACHE_ENTRY_LIMIT, Cache, IMAGE_CACHE_LIMIT, LocalImageLoader, RequestKey, egui};

fn image_bytes(image: &Result<egui::ColorImage, String>) -> usize {
    image
        .as_ref()
        .map_or(0, LocalImageLoader::color_image_bytes)
}

fn result_bytes(image: &Result<Arc<egui::ColorImage>, String>) -> usize {
    image
        .as_ref()
        .map_or(0, |image| LocalImageLoader::color_image_bytes(image))
}

impl LocalImageLoader {
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
        let cache = self.inner.cache.lock().ok()?;
        cache.ready.get(key).map(|image| match image {
            Ok(image) => super::LocalImageStatus::Ready(Arc::clone(image)),
            Err(error) => super::LocalImageStatus::Failed(error.clone()),
        })
    }

    pub(super) fn store_result(&self, key: RequestKey, image: Result<egui::ColorImage, String>) {
        let bytes = image_bytes(&image);
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        if bytes > IMAGE_CACHE_LIMIT {
            if let Some(old) = cache.ready.remove(&key) {
                cache.bytes = cache.bytes.saturating_sub(result_bytes(&old));
            }
            cache.textures.remove(&key);
            cache
                .ready
                .insert(key, Err("image exceeds cache limit".to_owned()));
            trim_cache(&mut cache);
            return;
        }
        cache.bytes = cache
            .bytes
            .saturating_sub(cache.ready.get(&key).map_or(0, result_bytes));
        cache.bytes += bytes;
        cache.ready.insert(key, image.map(Arc::new));
        trim_cache(&mut cache);
    }

    pub(super) fn store_texture(&self, key: RequestKey, texture: egui::TextureHandle) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.textures.insert(key, texture);
        trim_cache(&mut cache);
    }
}

fn trim_cache(cache: &mut Cache) {
    while (cache.bytes > IMAGE_CACHE_LIMIT || cache.ready.len() > CACHE_ENTRY_LIMIT)
        && cache.ready.len() > 1
    {
        let Some(key) = cache.ready.keys().next().cloned() else {
            break;
        };
        if let Some(old) = cache.ready.remove(&key) {
            cache.bytes = cache.bytes.saturating_sub(result_bytes(&old));
        }
        cache.textures.remove(&key);
    }
}
