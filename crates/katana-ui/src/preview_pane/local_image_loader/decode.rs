use super::super::cancellable_read::{CancellableReader, ReadCancellation};
use eframe::egui;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(super) struct ImageDecodeOps;

impl ImageDecodeOps {
    pub(super) fn load_image<F: Fn() -> bool>(
        path: &Path,
        background: egui::Color32,
        cancellation: &ReadCancellation<F>,
    ) -> Result<egui::ColorImage, String> {
        cancellation.check().map_err(|error| error.to_string())?;
        let file = File::open(path).map_err(|error| error.to_string())?;
        let capacity = file
            .metadata()
            .ok()
            .filter(std::fs::Metadata::is_file)
            .and_then(|metadata| usize::try_from(metadata.len()).ok());
        cancellation.check().map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        if let Some(capacity) = capacity {
            bytes
                .try_reserve(capacity)
                .map_err(|error| error.to_string())?;
        }
        let mut reader = CancellableReader::new(file, cancellation);
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        drop(reader);
        cancellation.check().map_err(|error| error.to_string())?;
        if is_svg(path) {
            let svg = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
            cancellation.check().map_err(|error| error.to_string())?;
            let rasterized =
                katana_core::markdown::svg_rasterize::SvgRasterizeOps::rasterize_svg(svg, 1.0)
                    .map_err(|error| error.to_string())?;
            cancellation.check().map_err(|error| error.to_string())?;
            let mut pixels = rasterized.rgba;
            super::super::image_background::ImageBackgroundOps::composite_rgba_over_background(
                &mut pixels,
                background,
            );
            cancellation.check().map_err(|error| error.to_string())?;
            return Ok(egui::ColorImage::from_rgba_unmultiplied(
                [rasterized.width as usize, rasterized.height as usize],
                &pixels,
            ));
        }
        cancellation.check().map_err(|error| error.to_string())?;
        let rgba = image::load_from_memory(&bytes)
            .map_err(|error| error.to_string())?
            .into_rgba8();
        cancellation.check().map_err(|error| error.to_string())?;
        let size = [rgba.width() as usize, rgba.height() as usize];
        let mut pixels = rgba.into_raw();
        super::super::image_background::ImageBackgroundOps::composite_rgba_over_background(
            &mut pixels,
            background,
        );
        cancellation.check().map_err(|error| error.to_string())?;
        Ok(egui::ColorImage::from_rgba_unmultiplied(size, &pixels))
    }
}

fn is_svg(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}
