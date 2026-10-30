use super::ImageLogicOps;
use crate::preview_pane::{ViewerState, local_image_loader::LocalImageLoader};
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const OPAQUE_RED: image::Rgba<u8> = image::Rgba([u8::MAX, 0, 0, u8::MAX]);
const READY_TEXTURE_TIMEOUT_SECS: u64 = 5;

#[derive(Clone, Copy)]
struct LayoutBounds {
    image: egui::Rect,
    selection: egui::Rect,
    following: egui::Rect,
    cursor_y: f32,
    texture: egui::TextureId,
}

#[test]
fn local_png_wide_image_controls_follow_image_and_selection() {
    assert_local_png_layout([640, 24], egui::vec2(800.0, 480.0));
}

#[test]
fn local_png_small_image_controls_follow_image_and_selection() {
    assert_local_png_layout([16, 12], egui::vec2(800.0, 480.0));
}

#[test]
fn local_png_controls_follow_image_in_narrow_viewport() {
    assert_local_png_layout([640, 24], egui::vec2(160.0, 480.0));
}

fn assert_local_png_layout(size: [u32; 2], viewport: egui::Vec2) {
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("PNG fixture directory");
    let path = directory.path().join("layout.png");
    image::RgbaImage::from_pixel(size[0], size[1], OPAQUE_RED)
        .save(&path)
        .expect("write PNG fixture");
    let recorded = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&recorded);
    let loader = LocalImageLoader::default();
    let mut state = ViewerState::default();
    let mut frame = 0;
    let mut harness = Harness::builder().with_size(viewport).build_ui(move |ui| {
        loader.poll(frame);
        frame += 1;
        let mut image_bounds = None;
        let selection = ImageLogicOps::show_local_image(
            ui,
            &path,
            0,
            &loader,
            Some(&mut state),
            None,
            |_, rect, _| image_bounds = Some(rect),
        );
        if let (Some(image), Some(selection), Some(texture)) =
            (image_bounds, selection, state.texture.as_ref())
        {
            let cursor_y = ui.cursor().top();
            let following = ui.separator().rect;
            *observed.lock().expect("PNG layout lock") = Some(LayoutBounds {
                image,
                selection,
                following,
                cursor_y,
                texture: texture.id(),
            });
        }
    });
    let deadline = Instant::now() + Duration::from_secs(READY_TEXTURE_TIMEOUT_SECS);
    /* WHY: 実ファイルの非同期デコードと通知をフレームで回収し、固定待機やデコードの代替を使わない。 */
    while recorded.lock().expect("PNG layout lock").is_none() && Instant::now() < deadline {
        harness.step();
        std::thread::yield_now();
    }
    let bounds = recorded
        .lock()
        .expect("PNG layout lock")
        .expect("real PNG texture must become ready within the GUI test budget");
    let painted = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Mesh(mesh) if mesh.texture_id == bounds.texture => {
                Some(mesh.calc_bounds())
            }
            _ => None,
        })
        .expect("ready PNG texture must be painted by the real caller");
    let scale = (bounds.image.width() / size[0] as f32).min(1.0);
    assert!((painted.width() - size[0] as f32 * scale).abs() < 0.01);
    assert!((painted.height() - size[1] as f32 * scale).abs() < 0.01);
    assert!((painted.center().x - bounds.image.center().x).abs() < 0.01);
    assert!(bounds.image.contains_rect(painted));
    assert!(
        bounds.cursor_y >= bounds.selection.bottom(),
        "cursor y={}, selection {:?}",
        bounds.cursor_y,
        bounds.selection
    );
    assert!(
        bounds.following.top() >= bounds.selection.bottom(),
        "following {:?}, selection {:?}",
        bounds.following,
        bounds.selection
    );
    assert_actual_control_bounds(&harness, bounds);
}

fn assert_actual_control_bounds(harness: &Harness<'_>, bounds: LayoutBounds) {
    let messages = crate::i18n::I18nOps::get();
    let controls = &messages.preview.diagram_controller;
    let fullscreen = harness.get_by_label(&controls.fullscreen).rect();
    assert!(bounds.image.contains_rect(fullscreen));
    assert!((bounds.image.right() - fullscreen.right() - 8.0).abs() <= 1.0);
    assert!((fullscreen.top() - bounds.image.top() - 8.0).abs() <= 1.0);
    let mut control_bounds = egui::Rect::NOTHING;
    for label in [
        &controls.pan_up,
        &controls.zoom_in,
        &controls.pan_left,
        &controls.reset,
        &controls.pan_right,
        &controls.trackpad_help,
        &controls.pan_down,
        &controls.zoom_out,
    ] {
        let button = harness.get_by_label(label).rect();
        assert!(
            button.top() >= bounds.image.bottom(),
            "control {label}: {button:?}, image {:?}",
            bounds.image
        );
        assert_eq!(bounds.image.intersect(button).area(), 0.0);
        assert!(bounds.selection.contains_rect(button));
        control_bounds = control_bounds.union(button);
    }
    assert!((control_bounds.right() - fullscreen.right()).abs() <= 1.0);
    assert!(bounds.selection.bottom() > bounds.image.bottom());
}
