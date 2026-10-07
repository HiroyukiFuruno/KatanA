use super::*;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;

fn send_result(
    loader: &LocalImageLoader,
    key: RequestKey,
    image: Result<egui::ColorImage, String>,
) {
    let generation = loader.generation();
    let revision = loader.path_revision(&key.path);
    loader
        .inner
        .result_tx
        .send(ResultMessage {
            key,
            generation,
            revision,
            image,
        })
        .expect("test result receiver must remain connected");
}

fn small_image() -> egui::ColorImage {
    egui::ColorImage::new([1, 1], vec![TEST_BACKGROUND])
}

fn active_image_len(loader: &LocalImageLoader) -> usize {
    loader
        .inner
        .cache
        .lock()
        .expect("cache lock")
        .active_images
        .len()
}

#[test]
fn poll_batch_keeps_small_results_until_first_texture_and_next_frame() {
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let mut keys = Vec::new();
    for index in 0..=CACHE_ENTRY_LIMIT {
        let key = RequestKey {
            path: PathBuf::from(format!("batch-small-{index}.png")),
            background: TEST_BACKGROUND,
        };
        send_result(&loader, key.clone(), Ok(small_image()));
        keys.push(key);
    }
    assert!(loader.poll(1));
    assert_eq!(active_image_len(&loader), CACHE_ENTRY_LIMIT + 1);
    let mut textures = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        let LocalTextureStatus::Ready(texture) =
            loader.texture(&context, &key.path, key.background, index)
        else {
            panic!("every result in one poll batch must be displayable once")
        };
        textures.push(texture.id());
    }
    assert_eq!(active_image_len(&loader), 0);
    loader.poll(2);
    for (index, (key, expected)) in keys.iter().zip(textures).enumerate() {
        let LocalTextureStatus::Ready(texture) =
            loader.texture(&context, &key.path, key.background, index)
        else {
            panic!("displayed small texture must survive the next frame")
        };
        assert_eq!(texture.id(), expected);
    }
    let cache = loader.inner.cache.lock().expect("cache lock");
    assert!(cache.ready.len() <= CACHE_ENTRY_LIMIT);
    assert!(cache.bytes <= IMAGE_CACHE_LIMIT);
    assert_eq!(cache.active_textures.len(), CACHE_ENTRY_LIMIT + 1);
}

#[test]
fn unconsumed_small_result_expires_after_frame_grace() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("unconsumed-small.png"),
        background: TEST_BACKGROUND,
    };
    send_result(&loader, key, Ok(small_image()));
    assert!(loader.poll(1));
    loader.poll(2);
    loader.poll(3);
    assert_eq!(active_image_len(&loader), 0);
}

#[test]
fn poll_batch_keeps_unconsumed_failures_without_requeueing() {
    let loader = LocalImageLoader::default();
    let mut keys = Vec::new();
    for index in 0..=CACHE_ENTRY_LIMIT {
        let key = RequestKey {
            path: PathBuf::from(format!("batch-failure-{index}.png")),
            background: TEST_BACKGROUND,
        };
        send_result(&loader, key.clone(), Err("decode failed".to_owned()));
        keys.push(key);
    }
    assert!(loader.poll(1));
    for key in &keys {
        let LocalImageStatus::Failed(error) = loader.request(&key.path, key.background) else {
            panic!("every failure in one poll batch must remain observable")
        };
        assert_eq!(error, "decode failed");
    }
    assert!(
        loader
            .inner
            .pending
            .lock()
            .expect("pending lock")
            .is_empty()
    );
}

#[test]
fn reset_discards_small_active_result() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("reset-small.png"),
        background: TEST_BACKGROUND,
    };
    send_result(&loader, key, Ok(small_image()));
    assert!(loader.poll(1));
    loader.reset();
    assert_eq!(active_image_len(&loader), 0);
    assert!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .ready
            .is_empty()
    );
}

#[test]
fn invalidate_path_discards_small_active_result() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("invalidate-small.png"),
        background: TEST_BACKGROUND,
    };
    send_result(&loader, key.clone(), Ok(small_image()));
    assert!(loader.poll(1));
    loader.invalidate_path(&key.path);
    assert_eq!(active_image_len(&loader), 0);
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
