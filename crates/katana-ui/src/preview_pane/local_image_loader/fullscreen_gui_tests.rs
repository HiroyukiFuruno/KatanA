use crate::preview_pane::{ImageLogicOps, PreviewPane, ViewerState};
use eframe::egui;
use std::path::Path;
use std::time::{Duration, Instant};

const IMAGE_WAIT_SECONDS: u64 = 5;
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];

fn show_regular_image(pane: &mut PreviewPane, context: &egui::Context, path: &Path) {
    let mut output = context.run_ui(Default::default(), |ui| {
        ImageLogicOps::show_local_image(
            ui,
            path,
            0,
            &pane.local_image_loader,
            Some(&mut pane.viewer_states[0]),
            None,
            |_, _, _| {},
        );
    });
    output.textures_delta.clear();
}

fn wait_texture(pane: &mut PreviewPane, context: &egui::Context, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(IMAGE_WAIT_SECONDS);
    while pane.viewer_states[0].texture.is_none() && Instant::now() < deadline {
        pane.local_image_loader.poll(0);
        show_regular_image(pane, context, path);
        std::thread::yield_now();
    }
    assert!(pane.viewer_states[0].texture.is_some());
}

fn overwrite_and_wait(pane: &mut PreviewPane, context: &egui::Context, path: &Path) {
    let initial = pane.viewer_states[0].texture_identity;
    image::RgbaImage::from_pixel(1, 1, image::Rgba(GREEN_PIXEL))
        .save(path)
        .expect("overwrite PNG");
    let deadline = Instant::now() + Duration::from_secs(IMAGE_WAIT_SECONDS);
    while Instant::now() < deadline {
        pane.local_image_loader.poll(0);
        show_regular_image(pane, context, path);
        if pane.viewer_states[0].texture_identity != initial
            && pane.viewer_states[0].texture.is_some()
        {
            return;
        }
        std::thread::yield_now();
    }
    panic!("regular preview must reload after fullscreen closes");
}

#[test]
fn fullscreen_close_preserves_regular_image_watch_and_reload() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("image.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("initial PNG");
    let mut pane = PreviewPane::default();
    let context = egui::Context::default();
    pane.viewer_states.push(ViewerState::default());
    wait_texture(&mut pane, &context, &path);
    let generation = pane.local_image_loader.generation();
    let texture = pane.viewer_states[0]
        .texture
        .as_ref()
        .expect("texture")
        .id();
    pane.fullscreen_image = Some(0);
    pane.fullscreen_viewer_state.texture = pane.viewer_states[0].texture.clone();
    pane.apply_fullscreen_result(None, Some(&context));
    assert!(pane.fullscreen_viewer_state.texture.is_none());
    assert_eq!(pane.local_image_loader.generation(), generation);
    assert_eq!(
        pane.viewer_states[0]
            .texture
            .as_ref()
            .expect("texture")
            .id(),
        texture
    );
    overwrite_and_wait(&mut pane, &context, &path);
    assert_ne!(
        pane.viewer_states[0]
            .texture
            .as_ref()
            .expect("new texture")
            .id(),
        texture
    );
}
