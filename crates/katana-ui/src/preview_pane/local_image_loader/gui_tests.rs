use super::*;
use crate::preview_pane::{ImageLogicOps, ViewerState};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

fn show_image(
    context: &egui::Context,
    loader: &LocalImageLoader,
    path: &std::path::Path,
    state: &mut ViewerState,
) {
    let mut output = context.run_ui(Default::default(), |ui| {
        let _ =
            ImageLogicOps::show_local_image(ui, path, 0, loader, Some(state), None, |_, _, _| {});
    });
    output.textures_delta.clear();
}

fn settle_initial_repaints(context: &egui::Context) {
    const MAX_SETTLE_FRAMES: usize = 8;
    /* WHY: 初期texture取得だけでは登録通知などの再描画要求が残るため、idle通知の計測前に実frameで消費する。 */
    for _ in 0..MAX_SETTLE_FRAMES {
        if !context.has_requested_repaint() {
            return;
        }
        let mut output = context.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
    }
    assert!(
        !context.has_requested_repaint(),
        "initial egui repaint did not settle across frames"
    );
}

#[test]
fn initial_repaint_is_consumed_before_idle_wakeup_measurement() {
    let context = egui::Context::default();
    assert!(context.has_requested_repaint());
    settle_initial_repaints(&context);
    assert!(!context.has_requested_repaint());
}

#[test]
fn viewer_state_entry_repaints_and_replaces_texture_after_atomic_write() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("image.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let mut state = ViewerState::default();
    let deadline = Instant::now() + Duration::from_secs(5);
    while state.texture.is_none() && Instant::now() < deadline {
        loader.poll(0);
        show_image(&context, &loader, &path, &mut state);
        std::thread::yield_now();
    }
    let initial_identity = state.texture_identity.expect("initial viewer texture");
    let initial_texture = state.texture.as_ref().expect("initial texture").id();
    settle_initial_repaints(&context);
    assert!(
        !context.has_requested_repaint(),
        "initial image must settle before testing an idle GUI wakeup: {}",
        super::watcher_overflow_tests::watch_state(&loader, &path)
    );
    let repaint = Arc::new(AtomicBool::new(false));
    let callback_repaint = Arc::clone(&repaint);
    context.set_request_repaint_callback(move |_| {
        callback_repaint.store(true, Ordering::Release);
    });
    repaint.store(false, Ordering::Release);

    let replacement = directory.path().join("replacement.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 255, 0, 255]))
        .save(&replacement)
        .expect("replacement PNG");
    std::fs::rename(replacement, &path).expect("atomic replacement");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !repaint.load(Ordering::Acquire) && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        repaint.load(Ordering::Acquire),
        "idle watcher must wake the GUI: egui_pending={}, {}",
        context.has_requested_repaint(),
        super::watcher_overflow_tests::watch_state(&loader, &path)
    );
    while Instant::now() < deadline {
        loader.poll(0);
        show_image(&context, &loader, &path, &mut state);
        if state.texture_identity != Some(initial_identity) && state.texture.is_some() {
            break;
        }
        std::thread::yield_now();
    }
    assert_ne!(state.texture_identity, Some(initial_identity));
    assert_ne!(
        state.texture.as_ref().expect("replacement texture").id(),
        initial_texture
    );
}
