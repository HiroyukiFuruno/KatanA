use super::LOAD_TIMEOUT;
use super::fixture::ExplorerImageOps;
use eframe::egui;
use egui_kittest::Harness;
use std::path::Path;
use std::time::{Duration, Instant};

const STABLE_TEXTURE_FRAMES: usize = 10;

impl ExplorerImageOps {
    pub(super) fn assert_texture_painted(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
    ) {
        let elapsed =
            Self::assert_texture_painted_with_budget(harness, image_path, started, LOAD_TIMEOUT);
        eprintln!(
            "png_explorer_paint elapsed_ms={}",
            elapsed.as_secs_f64() * 1000.0
        );
    }
    pub(super) fn assert_texture_painted_with_budget(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
        budget: Duration,
    ) -> Duration {
        let texture = Self::wait_for_texture_paint(harness, image_path, started, budget);
        let elapsed = started.elapsed();
        Self::assert_texture_stable(harness, image_path, texture);
        elapsed
    }
    fn wait_for_texture_paint(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
        budget: Duration,
    ) -> egui::TextureId {
        let deadline = started + budget;
        loop {
            Self::step(harness);
            assert!(
                Instant::now() < deadline,
                "PNG never reached an epaint mesh within {}ms: elapsed_ms={}",
                budget.as_millis(),
                started.elapsed().as_secs_f64() * 1000.0
            );
            if let Some(texture) = Self::painted_texture(harness, image_path) {
                return texture;
            }
            std::thread::yield_now();
        }
    }
    pub(super) fn painted_texture(
        harness: &Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
    ) -> Option<egui::TextureId> {
        let texture = Self::local_image_texture(harness.state(), image_path)?;
        harness
            .output()
            .shapes
            .iter()
            .any(|shape| Self::shape_has_texture(&shape.shape, texture))
            .then_some(texture)
    }
    fn assert_texture_stable(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        texture: egui::TextureId,
    ) {
        for _ in 0..STABLE_TEXTURE_FRAMES {
            Self::step(harness);
            assert_eq!(
                Self::local_image_texture(harness.state(), image_path),
                Some(texture)
            );
            assert!(
                harness
                    .output()
                    .shapes
                    .iter()
                    .any(|shape| Self::shape_has_texture(&shape.shape, texture))
            );
        }
    }
    fn local_image_texture(app: &crate::shell::KatanaApp, path: &Path) -> Option<egui::TextureId> {
        app.tab_previews
            .iter()
            .find(|tab| tab.path == path)?
            .pane
            .viewer_states
            .iter()
            .find_map(|state| state.texture.as_ref().map(egui::TextureHandle::id))
    }
    fn shape_has_texture(shape: &egui::epaint::Shape, texture: egui::TextureId) -> bool {
        match shape {
            egui::epaint::Shape::Mesh(mesh) => mesh.texture_id == texture,
            egui::epaint::Shape::Vec(children) => children
                .iter()
                .any(|child| Self::shape_has_texture(child, texture)),
            _ => false,
        }
    }
}
