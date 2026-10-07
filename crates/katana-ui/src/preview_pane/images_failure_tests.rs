use crate::preview_pane::{ImageLogicOps, SectionImageOps, SectionLifecycle, ViewerState};
use eframe::egui;
use std::path::Path;
use std::time::{Duration, Instant};

const WAIT_SECONDS: u64 = 5;
const FIXTURE_PIXEL: image::Rgba<u8> = image::Rgba([255, 0, 0, 255]);

fn show_image(
    context: &egui::Context,
    loader: &super::local_image_loader::LocalImageLoader,
    path: &Path,
    mut state: Option<&mut ViewerState>,
) -> Option<egui::Rect> {
    let mut rect = None;
    let mut output = context.run_ui(Default::default(), |ui| {
        rect = ImageLogicOps::show_local_image(
            ui,
            path,
            0,
            loader,
            state.as_deref_mut(),
            None,
            |_, _, _| {},
        );
    });
    output.textures_delta.clear();
    rect
}

fn wait_for_failed(
    context: &egui::Context,
    loader: &super::local_image_loader::LocalImageLoader,
    path: &Path,
) {
    let background = super::image_background::ImageBackgroundOps::preview_background(
        context,
        context.global_style().visuals.window_fill(),
    );
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        if matches!(
            loader.request(path, background),
            super::local_image_loader::LocalImageStatus::Failed(_)
        ) {
            return;
        }
        std::thread::yield_now();
    }
    panic!("missing image must reach the Failed decode state");
}

#[test]
fn failed_local_image_keeps_rect_without_viewer_state() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("missing.png");
    let loader = super::local_image_loader::LocalImageLoader::default();
    let context = egui::Context::default();
    wait_for_failed(&context, &loader, &path);

    let rect = show_image(&context, &loader, &path, None).expect("failure label rect");
    assert!(rect.is_positive());
}

#[test]
fn failed_local_image_keeps_rect_with_viewer_state() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("missing-viewer.png");
    let loader = super::local_image_loader::LocalImageLoader::default();
    let context = egui::Context::default();
    wait_for_failed(&context, &loader, &path);
    let mut state = ViewerState::default();

    let rect = show_image(&context, &loader, &path, Some(&mut state)).expect("failure label rect");
    assert!(rect.is_positive());
    assert!(state.texture.is_none());
}

#[test]
fn pending_local_image_has_no_rect_before_decode() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("pending.png");
    image::RgbaImage::from_pixel(1, 1, FIXTURE_PIXEL)
        .save(&path)
        .expect("PNG fixture");
    let loader = super::local_image_loader::LocalImageLoader::default();
    let context = egui::Context::default();
    let mut state = ViewerState::default();

    assert!(show_image(&context, &loader, &path, Some(&mut state)).is_none());
}

#[test]
fn failed_section_image_marks_drawn_and_anchor() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("missing-section.png");
    let loader = super::local_image_loader::LocalImageLoader::default();
    let context = egui::Context::default();
    wait_for_failed(&context, &loader, &path);
    let mut lifecycle = vec![SectionLifecycle::default()];
    let mut lifecycle_ref = Some(&mut lifecycle);
    let mut anchors = Vec::new();
    let mut anchors_ref = Some(&mut anchors);
    let mut viewer_states = vec![ViewerState::default()];

    let mut output = context.run_ui(Default::default(), |ui| {
        SectionImageOps::handle_local_image_section(
            ui,
            &path,
            0,
            &loader,
            1,
            0,
            None,
            Some(&mut viewer_states),
            None,
            &mut lifecycle_ref,
            &mut anchors_ref,
            None,
            false,
        );
    });
    output.textures_delta.clear();

    assert!(lifecycle[0].is_drawn);
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].0, 0..1);
    assert!(anchors[0].1.is_positive());
}
