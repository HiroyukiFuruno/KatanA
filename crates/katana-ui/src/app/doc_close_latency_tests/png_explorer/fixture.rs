use super::super::fixtures::{CLOSE_TIMEOUT, await_workers_idle};
use super::{IMAGE_NAME, LOAD_TIMEOUT};
use crate::app::{DocumentOps, WorkspaceOps};
use crate::preview_pane::DocumentWorkerLifecycle;
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use std::path::{Path, PathBuf};
use std::time::Instant;
const NATIVE_TEST_MAX_TEXTURE_SIDE: usize = 8192;
const STABLE_TEXTURE_FRAMES: usize = 10;
pub(super) struct ExplorerImageOps;

impl ExplorerImageOps {
    pub(super) fn step(harness: &mut Harness<'_, crate::shell::KatanaApp>) {
        /* WHY: 生成した幅2886pxの画像をnative texture上限内で実描画する。 */
        harness.input_mut().max_texture_side = Some(NATIVE_TEST_MAX_TEXTURE_SIDE);
        harness.step();
    }

    pub(super) fn prepare_workspace(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        root: &Path,
    ) -> PathBuf {
        std::fs::create_dir_all(root).expect("create isolated Explorer workspace");
        let image_path = root.join(IMAGE_NAME);
        super::input::LoadingFixtureInputs::create_png(&image_path);
        harness.state_mut().state.layout.show_explorer = true;
        harness.state_mut().handle_open_explorer(root.to_path_buf());
        let deadline = Instant::now() + LOAD_TIMEOUT;
        while harness.state_mut().state.workspace.data.is_none() {
            assert!(
                Instant::now() < deadline,
                "Explorer workspace load exceeded 15 seconds"
            );
            Self::step(harness);
            std::thread::yield_now();
        }
        image_path
    }
    pub(super) fn click_png_row(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        office_path: Option<&Path>,
        office_baseline: usize,
    ) -> Instant {
        Self::step(harness);
        let started = Instant::now();
        harness.get_by_label(&format!("file {IMAGE_NAME}")).click();
        Self::await_png_selection(harness, image_path, started);
        assert_eq!(
            harness.state().state.active_path(),
            Some(image_path.to_path_buf())
        );
        if let Some(path) = office_path {
            Self::assert_office_inflight(harness, path, office_baseline);
        }
        eprintln!(
            "png_explorer_selection elapsed_ms={}",
            started.elapsed().as_secs_f64() * 1000.0
        );
        started
    }
    fn await_png_selection(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
    ) {
        let deadline = started + LOAD_TIMEOUT;
        while harness.state().state.active_path().as_deref() != Some(image_path) {
            assert!(
                Instant::now() < deadline,
                "Explorer click did not select the PNG within 15 seconds"
            );
            Self::step(harness);
            std::thread::yield_now();
        }
    }
    fn assert_office_inflight(
        harness: &Harness<'_, crate::shell::KatanaApp>,
        path: &Path,
        baseline: usize,
    ) {
        assert!(DocumentWorkerLifecycle::live_count() > baseline);
        let pane = harness
            .state()
            .tab_previews
            .iter()
            .find(|tab| tab.path == path)
            .expect("Office preview retained at PNG selection");
        assert!(pane.pane.document_frame_state_for_test().is_none());
    }
    pub(super) fn assert_texture_painted(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
    ) {
        let texture = Self::wait_for_texture_paint(harness, image_path, started);
        Self::assert_texture_stable(harness, image_path, texture);
        eprintln!(
            "png_explorer_paint elapsed_ms={}",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    fn wait_for_texture_paint(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        image_path: &Path,
        started: Instant,
    ) -> egui::TextureId {
        let deadline = started + LOAD_TIMEOUT;
        loop {
            Self::step(harness);
            let texture = Self::local_image_texture(harness.state(), image_path);
            if let Some(texture) = texture
                && harness
                    .output()
                    .shapes
                    .iter()
                    .any(|shape| Self::shape_has_texture(&shape.shape, texture))
            {
                return texture;
            }
            assert!(
                Instant::now() < deadline,
                "PNG never reached an epaint mesh within the 15-second load budget"
            );
            std::thread::yield_now();
        }
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
    pub(super) fn close_all(
        harness: &mut Harness<'_, crate::shell::KatanaApp>,
        baseline: usize,
        kind: &str,
    ) {
        let deadline = Instant::now() + CLOSE_TIMEOUT;
        while !harness.state().state.document.open_documents.is_empty() {
            let last = harness.state().state.document.open_documents.len() - 1;
            harness.state_mut().force_close_document(last);
            assert!(
                Instant::now() < deadline,
                "{kind} documents exceeded the 5-second close budget"
            );
        }
        Self::step(harness);
        await_workers_idle(harness, baseline, kind, deadline);
        assert_eq!(
            harness.state_mut().preview_resource_counts_for_test(),
            (0, 0, 0, 0, 0, 0)
        );
    }
}
