#[derive(Debug, Clone)]
pub struct RasterizedSvg {
    pub width: u32,
    pub height: u32,
    pub display_width: f32,
    pub display_height: f32,
    pub rgba: Vec<u8>,
}

pub struct SvgRasterizeOps;

#[derive(Debug, thiserror::Error)]
pub enum SvgRasterizeError {
    #[error("Failed to parse SVG: {0}")]
    ParseFailed(String),
}
