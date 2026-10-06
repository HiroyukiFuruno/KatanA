use super::*;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;

#[test]
fn small_displayed_textures_survive_reuse_cache_eviction() {
    let loader = LocalImageLoader::default();
    let context = egui::Context::default();
    let mut displayed = Vec::new();
    for index in 0..=CACHE_ENTRY_LIMIT {
        let key = RequestKey {
            path: PathBuf::from(format!("visible-small-{index}.png")),
            background: TEST_BACKGROUND,
        };
        let image = egui::ColorImage::new([1, 1], vec![TEST_BACKGROUND]);
        loader.store_result(key.clone(), Ok(image.clone()));
        /* WHY: 再利用cacheの任意退避とは独立に、実描画済みtextureの所有期間を検証する。 */
        let LocalTextureStatus::Ready(texture) =
            loader.build_texture(&context, key.clone(), Arc::new(image), index)
        else {
            panic!("first display must produce a texture")
        };
        displayed.push((key, texture.id()));
    }
    loader.poll(1);
    for (index, (key, expected)) in displayed.iter().enumerate() {
        let LocalTextureStatus::Ready(texture) =
            loader.texture(&context, &key.path, key.background, index)
        else {
            panic!("visible small texture must survive reusable-cache eviction")
        };
        assert_eq!(texture.id(), *expected);
    }
    {
        let cache = loader.inner.cache.lock().expect("cache lock");
        assert!(cache.ready.len() <= CACHE_ENTRY_LIMIT);
        assert!(cache.textures.len() <= CACHE_ENTRY_LIMIT);
        assert!(cache.bytes <= IMAGE_CACHE_LIMIT);
        assert_eq!(cache.active_textures.len(), displayed.len());
    }
    loader.poll(2);
    assert_eq!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .len(),
        displayed.len()
    );
    loader.poll(3);
    assert!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .is_empty()
    );
    loader.reset();
    let cache = loader.inner.cache.lock().expect("cache lock");
    assert!(cache.ready.is_empty());
    assert!(cache.textures.is_empty());
    assert!(cache.active_textures.is_empty());
}
