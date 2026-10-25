use super::*;
use std::time::Instant;

#[test]
fn failed_decode_backoff_attempt_counter_saturates() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("retry-counter.png"),
        background: crate::theme_bridge::WHITE,
    };
    loader.store_result(key.clone(), Err("first failure".to_owned()));
    let mut cache = loader.inner.cache.lock().expect("cache lock");
    let Some(Err(failure)) = cache.ready.get_mut(&key) else {
        panic!("decode retry");
    };
    failure.attempts = u32::MAX;
    drop(cache);
    loader.store_result(key.clone(), Err("second failure".to_owned()));
    let cache = loader.inner.cache.lock().expect("cache lock");
    let Some(Err(failure)) = cache.ready.get(&key) else {
        panic!("decode retry");
    };
    assert_eq!(failure.attempts, u32::MAX);
}

#[test]
fn rearm_updates_active_failure_after_ready_copy_is_removed() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("rearm-boundary.png"),
        background: crate::theme_bridge::WHITE,
    };
    loader.store_result(key.clone(), Err("decode failed".to_owned()));
    {
        let mut cache = loader.inner.cache.lock().expect("cache lock");
        if let Some(Err(failure)) = cache.ready.get_mut(&key) {
            failure.deadline = Instant::now();
        }
        if let Some((Err(failure), _)) = cache.active_images.get_mut(&key) {
            failure.deadline = Instant::now();
        }
    }
    loader.rearm_decode_failure(&key);
    loader
        .inner
        .cache
        .lock()
        .expect("cache lock")
        .ready
        .remove(&key);
    assert!(matches!(
        loader.cached_status(&key),
        Some(LocalImageStatus::Failed(_))
    ));
}
