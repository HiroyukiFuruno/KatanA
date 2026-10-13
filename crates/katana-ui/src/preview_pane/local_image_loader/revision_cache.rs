use super::LocalImageLoader;
use eframe::egui;
use std::path::Path;
use std::sync::Arc;

impl LocalImageLoader {
    pub(super) fn invalidate_path(&self, path: &Path) {
        let Ok(mut paths) = self.inner.watched_paths.lock() else {
            return;
        };
        paths.remove(path);
        drop(paths);
        let Ok(mut revisions) = self.inner.path_revisions.lock() else {
            return;
        };
        let revision = revisions.entry(path.to_path_buf()).or_default();
        *revision = revision.wrapping_add(1);
        drop(revisions);
        let Ok(mut pending) = self.inner.pending.lock() else {
            return;
        };
        pending.retain(|key| key.path != path);
        drop(pending);
        self.invalidate_cached_path(path);
    }

    fn invalidate_cached_path(&self, path: &Path) {
        let Ok(mut cache) = self.inner.cache.lock() else {
            return;
        };
        cache.ready.retain(|key, _| key.path != path);
        cache.textures.retain(|key, _| key.path != path);
        cache.active_images.retain(|key, _| key.path != path);
        cache.active_textures.retain(|key, _| key.path != path);
        cache.bytes = cache.ready.values().map(result_bytes).sum();
    }
}

fn result_bytes(image: &Result<Arc<egui::ColorImage>, String>) -> usize {
    image.as_ref().map_or(0, |image| {
        image.pixels.len() * std::mem::size_of::<egui::Color32>()
    })
}
