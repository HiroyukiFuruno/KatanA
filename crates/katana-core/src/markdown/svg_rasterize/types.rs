#[derive(Debug, Clone)]
pub struct RasterizedSvg {
    pub width: u32,
    pub height: u32,
    pub display_width: f32,
    pub display_height: f32,
    pub content_hash: u64,
    pub rgba: Vec<u8>,
}

impl RasterizedSvg {
    pub fn new(
        width: u32,
        height: u32,
        display_width: f32,
        display_height: f32,
        rgba: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            display_width,
            display_height,
            content_hash: stable_hash_bytes(&rgba),
            rgba,
        }
    }
}

pub struct SvgRasterizeOps;

#[derive(Debug, thiserror::Error)]
pub enum SvgRasterizeError {
    #[error("Failed to parse SVG: {0}")]
    ParseFailed(String),
}

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    bytes.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}
