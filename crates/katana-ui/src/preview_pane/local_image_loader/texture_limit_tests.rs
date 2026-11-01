use super::{Arc, LocalImageTexture, egui};

#[test]
fn repository_png_that_crashed_startup_fits_the_real_device_limit() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/candidates/katana/legacy-krr-0.4.19/export_png/sample.png");
    let cancellation = super::super::cancellable_read::ReadCancellation::new(|| false);
    let image = super::decode::ImageDecodeOps::load_image(
        &path,
        crate::theme_bridge::WHITE,
        &cancellation,
        None,
    )
    .expect("decode the actual reported PNG");
    assert_eq!(image.size, [1280, 19067]);
    let context = egui::Context::default();
    context.input_mut(|input| input.max_texture_side = 8192);
    let texture = LocalImageTexture::load(&context, "startup-regression".into(), Arc::new(image));
    assert_eq!(texture.size(), [549, 8192]);
    assert_eq!(texture.source_size, egui::vec2(1280.0, 19067.0));
    let mut delta = context.tex_manager().write().take_delta();
    let upload = delta.set.get(&texture.id()).unwrap().last().unwrap();
    assert_eq!(upload.image.size(), [549, 8192]);
    delta.clear();
}

#[test]
fn fitting_preserves_premultiplied_alpha_and_original_source_size() {
    let context = egui::Context::default();
    context.input_mut(|input| input.max_texture_side = 32);
    let sample = egui::ColorImage::from_rgba_unmultiplied([1, 1], &[160, 80, 40, 128]);
    let pixel = sample.pixels[0];
    let source = Arc::new(egui::ColorImage::filled([2, 64], pixel));
    let texture = LocalImageTexture::load(&context, "alpha-regression".into(), source);
    assert_eq!(texture.size(), [1, 32]);
    assert_eq!(texture.source_size, egui::vec2(2.0, 64.0));
    let mut delta = context.tex_manager().write().take_delta();
    let upload = delta.set.get(&texture.id()).unwrap().last().unwrap();
    let egui::ImageData::Color(image) = &upload.image;
    assert!(image.pixels.iter().all(|actual| *actual == pixel));
    delta.clear();
}
