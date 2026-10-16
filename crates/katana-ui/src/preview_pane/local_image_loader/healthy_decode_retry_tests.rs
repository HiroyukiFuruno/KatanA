use super::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const DECODE_TIMEOUT: Duration = Duration::from_secs(5);
fn request_key(name: &str) -> RequestKey {
    RequestKey {
        path: PathBuf::from(name),
        background: BACKGROUND,
    }
}

fn force_due(loader: &LocalImageLoader, key: &RequestKey) {
    let mut cache = loader.inner.cache.lock().expect("cache lock");
    let mut found = false;
    if let Some(Err(failure)) = cache.ready.get_mut(key) {
        failure.deadline = Instant::now();
        found = true;
    }
    if let Some((Err(failure), _)) = cache.active_images.get_mut(key) {
        failure.deadline = Instant::now();
        found = true;
    }
    assert!(found, "decode retry");
}

fn failure_attempts(loader: &LocalImageLoader, key: &RequestKey) -> u32 {
    let cache = loader.inner.cache.lock().expect("cache lock");
    cache
        .ready
        .get(key)
        .and_then(|result| result.as_ref().err())
        .or_else(|| {
            cache
                .active_images
                .get(key)
                .and_then(|(result, _)| result.as_ref().err())
        })
        .expect("decode retry")
        .attempts
}

fn wait_status(loader: &LocalImageLoader, path: &Path) -> LocalImageStatus {
    let deadline = Instant::now() + DECODE_TIMEOUT;
    let mut frame = 0;
    while Instant::now() < deadline {
        loader.poll(frame);
        let status = loader.request(path, BACKGROUND);
        if !matches!(&status, LocalImageStatus::Pending) {
            return status;
        }
        frame += 1;
        std::thread::yield_now();
    }
    LocalImageStatus::Pending
}

#[test]
fn healthy_failed_cache_keeps_failure_until_deadline_and_repaints() {
    let loader = LocalImageLoader::default();
    let path = PathBuf::from("healthy-retry.png");
    let key = request_key("healthy-retry.png");
    loader.store_result(key, Err("decode failed".to_owned()));
    let context = egui::Context::default();
    let delay = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&delay);
    context.set_request_repaint_callback(move |info| {
        if info.delay > Duration::ZERO {
            *observed.lock().expect("delay lock") = Some(info.delay);
        }
    });
    loader.set_repaint_context(&context);
    assert!(loader.watch_error(&path).is_none());
    assert!(matches!(
        loader.request(&path, BACKGROUND),
        LocalImageStatus::Failed(_)
    ));
    let delay = delay.lock().expect("delay lock").expect("retry repaint");
    assert!(delay > Duration::ZERO);
    assert!(delay <= Duration::from_secs(2));
}

#[test]
fn healthy_failed_retry_backoff_and_metadata_cleanup_are_bounded() {
    let loader = LocalImageLoader::default();
    let path = PathBuf::from("healthy-backoff.png");
    let key = request_key("healthy-backoff.png");
    loader.store_result(key.clone(), Err("decode failed".to_owned()));
    force_due(&loader, &key);
    assert!(loader.cached_status(&key).is_none());
    loader.store_result(key.clone(), Err("decode failed again".to_owned()));
    assert_eq!(failure_attempts(&loader, &key), 1);
    loader.invalidate_path(&path);
    assert!(
        !loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .ready
            .contains_key(&key)
    );
}

#[test]
fn repeated_pending_lookup_preserves_decode_failure_backoff() {
    let loader = LocalImageLoader::default();
    let key = request_key("retry-pending.png");
    loader.store_result(key.clone(), Err("first failure".to_owned()));
    force_due(&loader, &key);
    assert!(loader.cached_status(&key).is_none());
    assert!(loader.cached_status(&key).is_none());
    loader.store_result(key.clone(), Err("second failure".to_owned()));
    assert_eq!(failure_attempts(&loader, &key), 1);
}

#[test]
fn failed_retry_metadata_is_bounded() {
    let capacity_loader = LocalImageLoader::default();
    for index in 0..(CACHE_ENTRY_LIMIT * 2) {
        let key = request_key(&format!("failure-{index}.png"));
        capacity_loader.store_result(key, Err("decode failed".to_owned()));
    }
    let cache = capacity_loader.inner.cache.lock().expect("cache lock");
    assert!(
        cache
            .ready
            .values()
            .filter(|result| result.is_err())
            .count()
            <= CACHE_ENTRY_LIMIT
    );
}

#[test]
fn failed_retry_is_background_specific() {
    let loader = LocalImageLoader::default();
    let path = PathBuf::from("backgrounds.png");
    let white = request_key("backgrounds.png");
    let black = RequestKey {
        path,
        background: crate::theme_bridge::BLACK,
    };
    loader.store_result(white.clone(), Err("white failed".to_owned()));
    loader.store_result(black.clone(), Err("black failed".to_owned()));
    force_due(&loader, &white);
    assert!(loader.cached_status(&white).is_none());
    assert!(matches!(
        loader.cached_status(&black),
        Some(LocalImageStatus::Failed(_))
    ));
}

#[test]
fn healthy_watcher_real_decode_failure_retries_to_ready() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("retry.png");
    std::fs::write(&path, b"invalid png").expect("invalid fixture");
    let loader = LocalImageLoader::default();
    assert!(matches!(
        wait_status(&loader, &path),
        LocalImageStatus::Failed(_)
    ));
    assert!(loader.watch_error(&path).is_none());
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&path)
        .expect("valid PNG");
    let key = RequestKey {
        path: path.clone(),
        background: BACKGROUND,
    };
    force_due(&loader, &key);
    assert!(matches!(
        loader.request(&path, BACKGROUND),
        LocalImageStatus::Pending
    ));
    let LocalImageStatus::Ready(image) = wait_status(&loader, &path) else {
        panic!("healthy watcher retry did not decode");
    };
    assert_eq!(image.size, [1, 1]);
    assert_eq!(image.pixels[0].to_array(), [255, 0, 0, 255]);
}
