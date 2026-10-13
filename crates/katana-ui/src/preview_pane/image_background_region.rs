use eframe::egui;

use super::image_background_region_model::{LightRegion, should_replace_light_region};

const RGBA_CHANNELS: usize = 4;
const ALPHA_CHANNEL_INDEX: usize = 3;
const SOLID_ALPHA_THRESHOLD: u8 = 250;
const LIGHT_CHANNEL_THRESHOLD: u8 = 245;

pub(super) struct ImageBackgroundRegionOps;

impl ImageBackgroundRegionOps {
    pub(super) fn replace_large_light_regions(
        rgba: &mut [u8],
        width: usize,
        height: usize,
        background: egui::Color32,
    ) {
        let total = width.saturating_mul(height);
        if total == 0 || rgba.len() < total.saturating_mul(RGBA_CHANNELS) {
            return;
        }

        let mut visited = vec![false; total];
        for index in 0..total {
            if visited[index] {
                continue;
            }
            visited[index] = true;
            if !is_near_white_pixel(rgba, index) {
                continue;
            }
            let region = Self::collect_region(rgba, width, height, index, &mut visited);
            if should_replace_light_region(&region, width, height, total) {
                for pixel_index in region.pixel_indices() {
                    set_pixel(rgba, pixel_index, background);
                }
            }
        }
    }

    fn collect_region(
        rgba: &[u8],
        width: usize,
        height: usize,
        start: usize,
        visited: &mut [bool],
    ) -> LightRegion {
        let mut region = LightRegion::new(start, width);
        let mut stack = vec![start];
        while let Some(index) = stack.pop() {
            region.include(index, width);
            Self::push_neighbors(rgba, width, height, index, visited, &mut stack);
        }
        region
    }

    fn push_neighbors(
        rgba: &[u8],
        width: usize,
        height: usize,
        index: usize,
        visited: &mut [bool],
        stack: &mut Vec<usize>,
    ) {
        let x = index % width;
        let y = index / width;
        if x > 0 {
            push_if_light(rgba, index - 1, visited, stack);
        }
        if x + 1 < width {
            push_if_light(rgba, index + 1, visited, stack);
        }
        if y > 0 {
            push_if_light(rgba, index - width, visited, stack);
        }
        if y + 1 < height {
            push_if_light(rgba, index + width, visited, stack);
        }
    }
}

fn push_if_light(rgba: &[u8], index: usize, visited: &mut [bool], stack: &mut Vec<usize>) {
    if visited[index] {
        return;
    }
    visited[index] = true;
    if is_near_white_pixel(rgba, index) {
        stack.push(index);
    }
}

fn is_near_white_pixel(rgba: &[u8], index: usize) -> bool {
    let start = index * RGBA_CHANNELS;
    rgba[start + ALPHA_CHANNEL_INDEX] >= SOLID_ALPHA_THRESHOLD
        && rgba[start] >= LIGHT_CHANNEL_THRESHOLD
        && rgba[start + 1] >= LIGHT_CHANNEL_THRESHOLD
        && rgba[start + 2] >= LIGHT_CHANNEL_THRESHOLD
}

fn set_pixel(rgba: &mut [u8], index: usize, background: egui::Color32) {
    let start = index * RGBA_CHANNELS;
    rgba[start] = background.r();
    rgba[start + 1] = background.g();
    rgba[start + 2] = background.b();
    rgba[start + ALPHA_CHANNEL_INDEX] = u8::MAX;
}

#[cfg(test)]
mod tests {
    use super::ImageBackgroundRegionOps;
    use super::RGBA_CHANNELS;

    #[test]
    fn replaces_large_light_region_but_keeps_small_enclosed_region() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let mut rgba = Vec::new();
        let background = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
            katana_platform::theme::ThemePreset::SolarizedLight
                .colors()
                .preview
                .background,
        );
        for index in 0..100 {
            let x = index % 10;
            let y = index / 10;
            let pixel = if (x == 7 && y == 7) || x < 6 {
                white
            } else {
                black
            };
            rgba.extend_from_slice(&pixel);
        }

        ImageBackgroundRegionOps::replace_large_light_regions(&mut rgba, 10, 10, background);

        assert_eq!(
            &rgba[0..RGBA_CHANNELS],
            &[background.r(), background.g(), background.b(), 255]
        );
        let enclosed_start = (7 * 10 + 7) * RGBA_CHANNELS;
        assert_eq!(
            &rgba[enclosed_start..enclosed_start + RGBA_CHANNELS],
            &white
        );
    }
}
