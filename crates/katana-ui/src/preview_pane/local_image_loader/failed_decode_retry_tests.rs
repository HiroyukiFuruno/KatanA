use super::*;
use std::time::{Duration, Instant};

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const WAIT_SECONDS: u64 = 5;

fn unavailable_watch(loader: &LocalImageLoader, path: &Path) {
    loader.store_watch_error(
        path.to_path_buf(),
        "watch unavailable".to_owned(),
        loader.generation(),
    );
    /* WHY: 登録未完了の単体状態を固定し、実decoderの再試行をwatch成功によるcache無効化と分離する。 */
    loader
        .inner
        .watch_pending
        .lock()
        .expect("pending watches")
        .insert(path.to_path_buf());
}

fn due_retry(loader: &LocalImageLoader, path: &Path) {
    loader
        .inner
        .watch_errors
        .lock()
        .expect("watch errors")
        .get_mut(path)
        .expect("watch failure")
        .deadline = Instant::now();
}

fn wait_decode_failure(loader: &LocalImageLoader, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        if matches!(
            loader.request(path, TEST_BACKGROUND),
            LocalImageStatus::Failed(_)
        ) {
            return;
        }
        std::thread::yield_now();
    }
    panic!("real decoder must report missing file");
}

#[test]
fn failed_decode_retries_at_watch_deadline_without_watch_success() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("created.png");
    let loader = LocalImageLoader::default();
    unavailable_watch(&loader, &path);
    wait_decode_failure(&loader, &path);
    image::RgbaImage::from_pixel(1, 1, image::Rgba(RED_PIXEL))
        .save(&path)
        .expect("created PNG");
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Failed(_)
    ));
    due_retry(&loader, &path);
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    super::watcher_tests::wait_ready(&loader, &path, TEST_BACKGROUND, RED_PIXEL);
    assert_eq!(
        loader.watch_error(&path).as_deref(),
        Some("watch unavailable")
    );
    assert!(
        !loader
            .inner
            .watched_paths
            .lock()
            .expect("watched paths")
            .contains(&path)
    );
}

#[test]
fn watch_retry_retains_ready_images_and_unrelated_failures() {
    let loader = LocalImageLoader::default();
    let path = Path::new("unit-unavailable-watch.png");
    let ready_key = RequestKey {
        path: path.to_path_buf(),
        background: TEST_BACKGROUND,
    };
    let failed_key = RequestKey {
        path: PathBuf::from("unit-other-failure.png"),
        background: TEST_BACKGROUND,
    };
    let same_path_failure = RequestKey {
        path: path.to_path_buf(),
        background: crate::theme_bridge::BLACK,
    };
    unavailable_watch(&loader, path);
    loader.store_result(
        ready_key.clone(),
        Ok(egui::ColorImage::new([1, 1], vec![egui::Color32::RED])),
    );
    loader.store_result(failed_key.clone(), Err("other decode failure".to_owned()));
    loader.store_result(
        same_path_failure.clone(),
        Err("same path failure".to_owned()),
    );
    let initial_bytes = loader.inner.cache.lock().expect("cache").bytes;
    let LocalImageStatus::Ready(initial) = loader.cached_status(&ready_key).expect("ready cache")
    else {
        panic!("ready image");
    };
    due_retry(&loader, path);
    let LocalImageStatus::Ready(retained) = loader.request(path, TEST_BACKGROUND) else {
        panic!("ready image retained");
    };
    assert!(Arc::ptr_eq(&initial, &retained));
    assert!(loader.cached_status(&same_path_failure).is_none());
    assert_eq!(
        loader.inner.cache.lock().expect("cache").bytes,
        initial_bytes
    );
    assert!(matches!(
        loader.cached_status(&failed_key),
        Some(LocalImageStatus::Failed(_))
    ));
    assert_eq!(
        loader.watch_error(path).as_deref(),
        Some("watch unavailable")
    );
}
