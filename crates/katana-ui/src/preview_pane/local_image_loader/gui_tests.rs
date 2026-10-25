use super::*;
use crate::preview_pane::{ImageLogicOps, ViewerState};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const READY_TEXTURE_TIMEOUT_SECS: u64 = 5;

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

fn show_fullscreen_image(
    context: &egui::Context,
    loader: &LocalImageLoader,
    path: &std::path::Path,
    state: &mut ViewerState,
) {
    let mut output = context.run_ui(Default::default(), |_| {
        crate::preview_pane::fullscreen_local::show_fullscreen_local(
            context, path, "close", state, 0, loader,
        );
    });
    output.textures_delta.clear();
}

fn seed_due_watch_failure(loader: &LocalImageLoader, path: &std::path::Path) {
    loader.store_watch_error(
        path.to_path_buf(),
        "watch unavailable".to_owned(),
        loader.generation(),
    );
    loader.clear_watch_state(path);
    loader
        .inner
        .watch_errors
        .lock()
        .expect("watch errors")
        .get_mut(path)
        .expect("watch failure")
        .deadline = Instant::now();
    loader
        .inner
        .watch_pending
        .lock()
        .expect("watch pending")
        .insert(path.to_path_buf());
}

fn retry_attempts(loader: &LocalImageLoader, path: &std::path::Path) -> u32 {
    loader
        .inner
        .watch_errors
        .lock()
        .expect("watch errors")
        .get(path)
        .expect("watch failure")
        .attempts
}

fn wait_ready_texture(
    loader: &LocalImageLoader,
    context: &egui::Context,
    path: &std::path::Path,
) -> egui::TextureHandle {
    let deadline = Instant::now() + Duration::from_secs(READY_TEXTURE_TIMEOUT_SECS);
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        if let LocalTextureStatus::Ready(texture) =
            loader.texture(context, path, crate::theme_bridge::WHITE, 0)
        {
            return texture;
        }
        frame += 1;
        std::thread::yield_now();
    }
    panic!("image texture did not become ready");
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

#[test]
fn cached_regular_viewer_texture_drives_watch_retry_without_replacement() {
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
    let initial = state.texture.as_ref().expect("initial texture").id();
    seed_due_watch_failure(&loader, &path);
    assert_eq!(retry_attempts(&loader, &path), 0);
    show_image(&context, &loader, &path, &mut state);
    assert_eq!(retry_attempts(&loader, &path), 1);
    assert_eq!(
        state.texture.as_ref().expect("cached texture").id(),
        initial
    );
}

#[test]
fn cached_fullscreen_viewer_texture_drives_watch_retry_without_replacement() {
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
        show_fullscreen_image(&context, &loader, &path, &mut state);
        std::thread::yield_now();
    }
    let initial = state.texture.clone().expect("initial texture");
    seed_due_watch_failure(&loader, &path);
    assert_eq!(retry_attempts(&loader, &path), 0);
    show_fullscreen_image(&context, &loader, &path, &mut state);
    assert_eq!(retry_attempts(&loader, &path), 1);
    assert_eq!(
        state.texture.as_ref().expect("cached texture").id(),
        initial.id()
    );
}

#[test]
fn cached_loader_texture_drives_watch_retry_without_replacement() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("image.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let initial = wait_ready_texture(&loader, &context, &path);
    seed_due_watch_failure(&loader, &path);
    assert_eq!(retry_attempts(&loader, &path), 0);
    let LocalTextureStatus::Ready(current) =
        loader.texture(&context, &path, crate::theme_bridge::WHITE, 0)
    else {
        panic!("cached texture must remain ready");
    };
    assert_eq!(retry_attempts(&loader, &path), 1);
    assert_eq!(current.id(), initial.id());
}

#[test]
fn active_loader_texture_drives_watch_retry_without_replacement() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("image.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("initial PNG");
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let initial = wait_ready_texture(&loader, &context, &path);
    let key = RequestKey {
        path: path.clone(),
        background: crate::theme_bridge::WHITE,
    };
    loader
        .inner
        .cache
        .lock()
        .expect("cache lock")
        .textures
        .remove(&key);
    {
        let cache = loader.inner.cache.lock().expect("cache lock");
        assert!(!cache.textures.contains_key(&key));
        assert_eq!(
            cache
                .active_textures
                .get(&key)
                .map(|(texture, _)| texture.id()),
            Some(initial.id())
        );
    }
    seed_due_watch_failure(&loader, &path);
    assert_eq!(retry_attempts(&loader, &path), 0);
    let LocalTextureStatus::Ready(current) =
        loader.texture(&context, &path, crate::theme_bridge::WHITE, 0)
    else {
        panic!("active texture must remain ready");
    };
    assert_eq!(retry_attempts(&loader, &path), 1);
    assert_eq!(current.id(), initial.id());
}
