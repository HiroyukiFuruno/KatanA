use super::{Arc, egui};

#[derive(Clone)]
pub(crate) struct LocalImageTexture {
    pub(crate) handle: egui::TextureHandle,
    pub(crate) source_size: egui::Vec2,
}

impl LocalImageTexture {
    pub(crate) fn load(
        context: &egui::Context,
        name: String,
        image: Arc<egui::ColorImage>,
    ) -> Self {
        let source_size = image.source_size;
        let limit = context.input(|input| input.max_texture_side);
        let fitted = fit_image(image, limit);
        Self {
            handle: context.load_texture(name, fitted, egui::TextureOptions::LINEAR),
            source_size,
        }
    }
}

impl std::ops::Deref for LocalImageTexture {
    type Target = egui::TextureHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

fn fit_image(image: Arc<egui::ColorImage>, limit: usize) -> Arc<egui::ColorImage> {
    let [width, height] = image.size;
    let longest = width.max(height);
    if longest <= limit {
        return image;
    }
    /* WHY: 元画像の表示寸法を保持し、GPUへ渡すpixelだけを実デバイスの上限内へ収める。 */
    let target_width = (width as u64 * limit as u64 / longest as u64).max(1) as u32;
    let target_height = (height as u64 * limit as u64 / longest as u64).max(1) as u32;
    let source = image::RgbaImage::from_fn(width as u32, height as u32, |x, y| {
        image::Rgba(image.pixels[y as usize * width + x as usize].to_array())
    });
    let resized = image::imageops::resize(
        &source,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );
    Arc::new(
        egui::ColorImage::from_rgba_premultiplied(
            [target_width as usize, target_height as usize],
            resized.as_raw(),
        )
        .with_source_size(image.source_size),
    )
}
