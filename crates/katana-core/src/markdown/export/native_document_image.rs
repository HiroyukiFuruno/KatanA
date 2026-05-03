use crate::markdown::MarkdownError;
use crate::markdown::svg_rasterize::RasterizedSvg;

const JPEG_QUALITY_PERCENT: u8 = 90;
const RGB_CHANNELS: usize = 3;
const ALPHA_CHANNEL_INDEX: usize = 3;
const MAX_ALPHA: u16 = 255;
const RGBA_BYTES: usize = 4;

pub(crate) struct NativeDocumentImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    rgba: Vec<u8>,
}

impl NativeDocumentImage {
    pub(crate) fn save_png(&self, output: &std::path::Path) -> Result<(), MarkdownError> {
        let image = self.rgba_image()?;
        image
            .save_with_format(output, image::ImageFormat::Png)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    pub(crate) fn save_jpeg(&self, output: &std::path::Path) -> Result<(), MarkdownError> {
        let bytes = self.jpeg_bytes()?;
        std::fs::write(output, bytes)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))
    }

    pub(crate) fn jpeg_bytes(&self) -> Result<Vec<u8>, MarkdownError> {
        let rgb = self.rgb_image()?;
        let mut bytes = Vec::new();
        let mut encoder =
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY_PERCENT);
        encoder
            .encode_image(&rgb)
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
        Ok(bytes)
    }

    fn rgba_image(&self) -> Result<image::RgbaImage, MarkdownError> {
        image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone()).ok_or_else(|| {
            MarkdownError::ExportFailed("native image buffer has invalid dimensions".to_string())
        })
    }

    fn rgb_image(&self) -> Result<image::RgbImage, MarkdownError> {
        let mut pixels =
            Vec::with_capacity((self.width * self.height * RGB_CHANNELS as u32) as usize);
        for chunk in self.rgba.chunks_exact(RGBA_BYTES) {
            let alpha = u16::from(chunk[ALPHA_CHANNEL_INDEX]);
            pixels.push(composite_over_white(chunk[0], alpha));
            pixels.push(composite_over_white(chunk[1], alpha));
            pixels.push(composite_over_white(chunk[2], alpha));
        }
        image::RgbImage::from_raw(self.width, self.height, pixels).ok_or_else(|| {
            MarkdownError::ExportFailed("native RGB buffer has invalid dimensions".to_string())
        })
    }
}

impl From<RasterizedSvg> for NativeDocumentImage {
    fn from(value: RasterizedSvg) -> Self {
        Self {
            width: value.width,
            height: value.height,
            rgba: value.rgba,
        }
    }
}

fn composite_over_white(value: u8, alpha: u16) -> u8 {
    (((u16::from(value) * alpha) + (MAX_ALPHA * (MAX_ALPHA - alpha))) / MAX_ALPHA) as u8
}
