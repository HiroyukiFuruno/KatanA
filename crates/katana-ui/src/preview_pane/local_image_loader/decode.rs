use eframe::egui;
use std::path::Path;

pub(super) fn load_image(
    path: &Path,
    background: egui::Color32,
) -> Result<egui::ColorImage, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    if is_svg(path) {
        let svg = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
        let rasterized =
            katana_core::markdown::svg_rasterize::SvgRasterizeOps::rasterize_svg(svg, 1.0)
                .map_err(|error| error.to_string())?;
        let mut pixels = rasterized.rgba;
        super::super::image_background::ImageBackgroundOps::composite_rgba_over_background(
            &mut pixels,
            background,
        );
        return Ok(egui::ColorImage::from_rgba_unmultiplied(
            [rasterized.width as usize, rasterized.height as usize],
            &pixels,
        ));
    }
    let rgba = image::load_from_memory(&bytes)
        .map_err(|error| error.to_string())?
        .into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let mut pixels = rgba.into_raw();
    super::super::image_background::ImageBackgroundOps::composite_rgba_over_background(
        &mut pixels,
        background,
    );
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, &pixels))
}

fn is_svg(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}
