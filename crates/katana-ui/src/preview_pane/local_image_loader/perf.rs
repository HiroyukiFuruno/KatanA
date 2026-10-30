use super::LocalImageLoader;
use crate::preview_pane::image_perf::{ImagePerf, ImageRequestTrace};
use std::path::Path;

impl LocalImageLoader {
    pub(super) fn perf_trace(&self, path: &Path) -> Option<ImageRequestTrace> {
        if !ImagePerf::enabled() {
            return None;
        }
        ImagePerf::request(
            std::sync::Arc::as_ptr(&self.inner) as usize,
            self.generation(),
            self.path_revision(path),
        )
    }

    pub(crate) fn trace_texture_queued(&self, path: &Path, texture: eframe::egui::TextureId) {
        ImagePerf::once(
            self.perf_trace(path),
            "texture_cpu_queued",
            path,
            Some(texture),
        );
    }

    pub(crate) fn trace_mesh_added(&self, path: &Path, texture: eframe::egui::TextureId) {
        ImagePerf::once(self.perf_trace(path), "mesh_added", path, Some(texture));
    }
}
