use super::{LocalImageLoader, LocalImageStatus, LocalTextureStatus, RequestKey, egui};
use std::path::Path;
use std::sync::atomic::Ordering;

impl LocalImageLoader {
    pub(crate) fn set_repaint_context(&self, context: &egui::Context) {
        let Ok(mut repaint_context) = self.inner.repaint_context.lock() else {
            return;
        };
        *repaint_context = Some(context.clone());
    }

    pub(crate) fn texture(
        &self,
        ctx: &egui::Context,
        path: &Path,
        background: egui::Color32,
        id: usize,
    ) -> LocalTextureStatus {
        self.set_repaint_context(ctx);
        let key = RequestKey {
            path: path.to_path_buf(),
            background,
        };
        if let Some(texture) = self.cached_texture(&key) {
            self.store_active_texture(key, texture.clone());
            return LocalTextureStatus::Ready(texture);
        }
        if let Some(texture) = self.active_texture(&key) {
            return LocalTextureStatus::Ready(texture.clone());
        }
        match self.request(path, background) {
            LocalImageStatus::Ready(image) => self.build_texture(ctx, key, image, id),
            LocalImageStatus::Pending => LocalTextureStatus::Pending,
            LocalImageStatus::Failed(error) => LocalTextureStatus::Failed(error),
        }
    }

    fn cached_texture(&self, key: &RequestKey) -> Option<egui::TextureHandle> {
        let cache = self.inner.cache.lock().ok()?;
        cache.textures.get(key).cloned()
    }

    fn active_texture(&self, key: &RequestKey) -> Option<egui::TextureHandle> {
        let mut cache = self.inner.cache.lock().ok()?;
        let (texture, seen) = cache.active_textures.get_mut(key)?;
        *seen = self.inner.active_frame.load(Ordering::Acquire);
        Some(texture.clone())
    }
}
