use super::*;
use std::time::Duration;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const LARGE_IMAGE_SIZE: [usize; 2] = [6000, 4000];

fn test_png_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "katana-local-image-{name}-{}-{}.png",
        std::process::id(),
        name.len()
    ))
}

fn wait_for(loader: &LocalImageLoader, path: &Path) -> LocalImageStatus {
    for frame in 0..100 {
        loader.poll(frame);
        let status = loader.request(path, TEST_BACKGROUND);
        if !matches!(status, LocalImageStatus::Pending) {
            return status;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    LocalImageStatus::Pending
}

#[test]
fn local_png_is_decoded_off_request_and_composited() {
    let path = test_png_path("decode");
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 128]));
    image.save(&path).expect("test PNG should be written");
    let loader = LocalImageLoader::default();
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    let status = wait_for(&loader, &path);
    let LocalImageStatus::Ready(image) = status else {
        match status {
            LocalImageStatus::Failed(error) => panic!("PNG decode failed: {error}"),
            LocalImageStatus::Pending => panic!("PNG decode remained pending"),
            LocalImageStatus::Ready(_) => unreachable!(),
        }
    };
    assert_eq!(image.size, [1, 1]);
    assert_ne!(
        image.pixels[0],
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128)
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn sixteen_bit_rgba_png_is_decoded_without_loss_of_dimensions() {
    let path = test_png_path("rgba16");
    let image: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
        image::ImageBuffer::from_pixel(2, 1, image::Rgba([u16::MAX, 32_768, 16_384, u16::MAX]));
    image.save(&path).expect("16-bit PNG should be written");
    let loader = LocalImageLoader::default();
    let status = wait_for(&loader, &path);
    let LocalImageStatus::Ready(image) = status else {
        panic!("16-bit PNG should load")
    };
    assert_eq!(image.size, [2, 1]);
    assert_eq!(image.pixels[0].to_array(), [255, 128, 64, 255]);
    let _ = std::fs::remove_file(path);
}

#[test]
fn failed_result_is_cached_without_retrying() {
    let path = test_png_path("missing");
    let loader = LocalImageLoader::default();
    let status = wait_for(&loader, &path);
    assert!(matches!(status, LocalImageStatus::Failed(_)));
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Failed(_)
    ));
}

#[test]
fn reset_discards_old_generation_result() {
    let path = test_png_path("generation");
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 255, 255]));
    image.save(&path).expect("test PNG should be written");
    let loader = LocalImageLoader::default();
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    loader.reset();
    for frame in 0..20 {
        loader.poll(frame);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    let _ = std::fs::remove_file(path);
}

#[test]
fn oversized_images_are_active_but_not_cached() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("oversized"),
        background: TEST_BACKGROUND,
    };
    loader.store_result(
        key.clone(),
        Ok(egui::ColorImage::new(
            LARGE_IMAGE_SIZE,
            vec![TEST_BACKGROUND; LARGE_IMAGE_SIZE[0] * LARGE_IMAGE_SIZE[1]],
        )),
    );
    let cache = loader.inner.cache.lock().expect("cache lock");
    assert!(!cache.ready.contains_key(&key));
    assert!(cache.active_images.contains_key(&key));
    drop(cache);
    let ctx = egui::Context::default();
    ctx.input_mut(|input| input.max_texture_side = 8192);
    let LocalTextureStatus::Ready(texture) = loader.texture(&ctx, &key.path, key.background, 1)
    else {
        panic!("oversized image should remain displayable")
    };
    assert_eq!(texture.size(), LARGE_IMAGE_SIZE);
    for frame in 1..5 {
        loader.poll(frame);
        loader.poll(frame);
        let LocalTextureStatus::Ready(current) = loader.texture(&ctx, &key.path, key.background, 1)
        else {
            panic!("displayed texture must not be decoded again")
        };
        assert_eq!(current.id(), texture.id());
    }
    let small_key = RequestKey {
        path: PathBuf::from("small"),
        background: TEST_BACKGROUND,
    };
    loader.store_result(
        small_key.clone(),
        Ok(egui::ColorImage::new([1, 1], vec![TEST_BACKGROUND])),
    );
    {
        let cache = loader.inner.cache.lock().expect("cache lock");
        assert!(!cache.active_images.contains_key(&key));
        assert!(cache.active_textures.contains_key(&key));
        assert!(cache.ready.contains_key(&small_key));
        assert!(cache.bytes <= IMAGE_CACHE_LIMIT);
    }
    loader.poll(5);
    assert!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .contains_key(&key)
    );
    loader.poll(6);
    assert!(
        !loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .contains_key(&key)
    );
    assert_eq!(texture.size(), LARGE_IMAGE_SIZE);
}

#[test]
fn active_display_entries_keep_same_frame_ownership_and_prune_afterward() {
    let loader = LocalImageLoader::default();
    let ctx = egui::Context::default();
    let mut textures = Vec::new();
    for index in 0..=CACHE_ENTRY_LIMIT {
        let key = RequestKey {
            path: PathBuf::from(format!("active-{index}")),
            background: TEST_BACKGROUND,
        };
        let texture = ctx.load_texture(
            format!("active-{index}"),
            egui::ColorImage::new([1, 1], vec![TEST_BACKGROUND]),
            egui::TextureOptions::LINEAR,
        );
        textures.push((key.clone(), texture.clone()));
        loader.store_active_texture(key, texture);
    }
    assert_eq!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .len(),
        CACHE_ENTRY_LIMIT + 1
    );
    for (key, expected) in &textures {
        let LocalTextureStatus::Ready(actual) = loader.texture(&ctx, &key.path, key.background, 1)
        else {
            panic!("same-frame active texture must remain available")
        };
        assert_eq!(actual.id(), expected.id());
    }
    loader.poll(1);
    assert_eq!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .len(),
        CACHE_ENTRY_LIMIT + 1
    );
    let next_key = RequestKey {
        path: PathBuf::from("active-next"),
        background: TEST_BACKGROUND,
    };
    let next_texture = ctx.load_texture(
        "active-next",
        egui::ColorImage::new([1, 1], vec![TEST_BACKGROUND]),
        egui::TextureOptions::LINEAR,
    );
    loader.store_active_texture(next_key.clone(), next_texture.clone());
    for (key, expected) in &textures {
        let LocalTextureStatus::Ready(actual) = loader.texture(&ctx, &key.path, key.background, 1)
        else {
            panic!("grace-frame active texture must remain available")
        };
        assert_eq!(actual.id(), expected.id());
    }
    let LocalTextureStatus::Ready(actual) =
        loader.texture(&ctx, &next_key.path, next_key.background, 1)
    else {
        panic!("new active texture must remain available")
    };
    assert_eq!(actual.id(), next_texture.id());
    loader.poll(2);
    assert_eq!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .active_textures
            .len(),
        CACHE_ENTRY_LIMIT + 2
    );
    loader.poll(3);
    let cache = loader.inner.cache.lock().expect("cache lock");
    assert!(cache.active_textures.is_empty());
    drop(cache);
    loader.reset();
    let cache = loader.inner.cache.lock().expect("cache lock");
    assert!(cache.active_images.is_empty());
    assert!(cache.active_textures.is_empty());
}
