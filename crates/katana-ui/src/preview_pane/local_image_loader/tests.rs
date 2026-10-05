use super::*;
use std::time::Duration;

const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;

fn test_png_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "katana-local-image-{name}-{}-{}.png",
        std::process::id(),
        name.len()
    ))
}

fn wait_for(loader: &LocalImageLoader, path: &Path) -> LocalImageStatus {
    for _ in 0..100 {
        loader.poll();
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
        panic!("PNG should load")
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
    for _ in 0..20 {
        loader.poll();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(matches!(
        loader.request(&path, TEST_BACKGROUND),
        LocalImageStatus::Pending
    ));
    let _ = std::fs::remove_file(path);
}

#[test]
fn cache_does_not_keep_oversized_images() {
    let loader = LocalImageLoader::default();
    let key = RequestKey {
        path: PathBuf::from("oversized"),
        background: TEST_BACKGROUND,
    };
    loader.store_result(
        key.clone(),
        Ok(egui::ColorImage::new(
            [IMAGE_CACHE_LIMIT / 4 + 1, 1],
            vec![TEST_BACKGROUND; IMAGE_CACHE_LIMIT / 4 + 1],
        )),
    );
    assert!(matches!(
        loader
            .inner
            .cache
            .lock()
            .expect("cache lock")
            .ready
            .get(&key),
        Some(Err(error)) if error == "image exceeds cache limit"
    ));
    assert!(matches!(
        loader.request(&key.path, key.background),
        LocalImageStatus::Failed(error) if error == "image exceeds cache limit"
    ));
}
