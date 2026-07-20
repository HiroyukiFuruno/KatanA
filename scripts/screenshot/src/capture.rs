use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRegion {
    pub min_x: u32,
    pub max_x: u32,
    pub min_y: u32,
    pub max_y: u32,
    pub center_x: u32,
    pub center_y: u32,
    pub pixels: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct PngBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn assert_png_changed(baseline: &Path, current: &Path, minimum: u64) -> Result<u64> {
    let baseline_image = image::open(baseline)?.to_rgba8();
    let current_image = image::open(current)?.to_rgba8();
    anyhow::ensure!(
        baseline_image.dimensions() == current_image.dimensions(),
        "screenshot dimensions differ: {} is {:?}, {} is {:?}",
        baseline.display(),
        baseline_image.dimensions(),
        current.display(),
        current_image.dimensions()
    );
    let changed = baseline_image
        .pixels()
        .zip(current_image.pixels())
        .filter(|(before, after)| before != after)
        .count() as u64;
    anyhow::ensure!(
        changed >= minimum,
        "screenshot change was too small: {changed} changed pixels, expected at least {minimum}"
    );
    Ok(changed)
}

pub fn assert_png_contains_rgb(
    screenshot: &Path,
    expected: [u8; 3],
    tolerance: u8,
    minimum: u64,
) -> Result<u64> {
    let image = image::open(screenshot)?.to_rgba8();
    let matching = image
        .pixels()
        .filter(|pixel| {
            pixel[3] > 0
                && pixel[0].abs_diff(expected[0]) <= tolerance
                && pixel[1].abs_diff(expected[1]) <= tolerance
                && pixel[2].abs_diff(expected[2]) <= tolerance
        })
        .count() as u64;
    anyhow::ensure!(
        matching >= minimum,
        "screenshot {} contains {matching} pixels near rgb({},{},{}), expected at least {minimum}",
        screenshot.display(),
        expected[0],
        expected[1],
        expected[2]
    );
    Ok(matching)
}

pub fn locate_largest_color_region_in_image(
    image: &image::RgbaImage,
    expected: [u8; 3],
    tolerance: u8,
    minimum: u64,
    search_bounds: Option<PngBounds>,
) -> Result<ColorRegion> {
    anyhow::ensure!(minimum > 0, "color region minimum must be positive");
    let bounds = color_search_bounds(image, search_bounds)?;
    let width = image.width();
    let height = image.height();
    let mut visited = vec![false; width as usize * height as usize];
    let mut largest = None;
    let search = MatchingColorSearch {
        image,
        expected,
        tolerance,
        width,
        bounds,
    };

    for start_y in bounds.min_y..bounds.max_y {
        for start_x in bounds.min_x..bounds.max_x {
            let start = pixel_index(width, start_x, start_y);
            if visited[start]
                || !matches_rgb(image.get_pixel(start_x, start_y), expected, tolerance)
            {
                continue;
            }
            let mut pending = vec![(start_x, start_y)];
            visited[start] = true;
            let mut pixels = 0_u64;
            let mut min_x = start_x;
            let mut max_x = start_x;
            let mut min_y = start_y;
            let mut max_y = start_y;

            while let Some((x, y)) = pending.pop() {
                pixels += 1;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                if x > bounds.min_x {
                    search.enqueue_neighbor(x - 1, y, &mut visited, &mut pending);
                }
                if x + 1 < bounds.max_x {
                    search.enqueue_neighbor(x + 1, y, &mut visited, &mut pending);
                }
                if y > bounds.min_y {
                    search.enqueue_neighbor(x, y - 1, &mut visited, &mut pending);
                }
                if y + 1 < bounds.max_y {
                    search.enqueue_neighbor(x, y + 1, &mut visited, &mut pending);
                }
            }

            if pixels >= minimum
                && largest
                    .as_ref()
                    .is_none_or(|current: &ColorRegion| pixels > current.pixels)
            {
                largest = Some(ColorRegion {
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    center_x: min_x + (max_x - min_x) / 2,
                    center_y: min_y + (max_y - min_y) / 2,
                    pixels,
                });
            }
        }
    }

    largest.context("rendered image has no matching RGB region")
}

#[derive(Debug, Clone, Copy)]
struct ColorSearchBounds {
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

fn color_search_bounds(
    image: &image::RgbaImage,
    requested: Option<PngBounds>,
) -> Result<ColorSearchBounds> {
    let (image_width, image_height) = image.dimensions();
    let requested = requested.unwrap_or(PngBounds {
        x: 0,
        y: 0,
        width: image_width,
        height: image_height,
    });
    let max_x = requested
        .x
        .checked_add(requested.width)
        .context("color search bounds overflow horizontally")?;
    let max_y = requested
        .y
        .checked_add(requested.height)
        .context("color search bounds overflow vertically")?;
    anyhow::ensure!(
        requested.width > 0
            && requested.height > 0
            && max_x <= image_width
            && max_y <= image_height,
        "color search bounds ({}, {}, {}, {}) are outside {}x{} image",
        requested.x,
        requested.y,
        requested.width,
        requested.height,
        image_width,
        image_height,
    );
    Ok(ColorSearchBounds {
        min_x: requested.x,
        max_x,
        min_y: requested.y,
        max_y,
    })
}

fn pixel_index(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}

struct MatchingColorSearch<'a> {
    image: &'a image::RgbaImage,
    expected: [u8; 3],
    tolerance: u8,
    width: u32,
    bounds: ColorSearchBounds,
}

impl MatchingColorSearch<'_> {
    fn enqueue_neighbor(
        &self,
        x: u32,
        y: u32,
        visited: &mut [bool],
        pending: &mut Vec<(u32, u32)>,
    ) {
        let index = pixel_index(self.width, x, y);
        if x < self.bounds.min_x
            || x >= self.bounds.max_x
            || y < self.bounds.min_y
            || y >= self.bounds.max_y
            || visited[index]
            || !matches_rgb(self.image.get_pixel(x, y), self.expected, self.tolerance)
        {
            return;
        }
        visited[index] = true;
        pending.push((x, y));
    }
}

fn matches_rgb(pixel: &image::Rgba<u8>, expected: [u8; 3], tolerance: u8) -> bool {
    pixel[3] > 0
        && pixel[0].abs_diff(expected[0]) <= tolerance
        && pixel[1].abs_diff(expected[1]) <= tolerance
        && pixel[2].abs_diff(expected[2]) <= tolerance
}

#[cfg(test)]
mod tests {
    use super::{
        assert_png_changed, assert_png_contains_rgb, locate_largest_color_region_in_image,
        PngBounds,
    };
    use image::{Rgba, RgbaImage};

    #[test]
    fn screenshot_change_assertion_counts_changed_pixels() {
        let root = tempfile::tempdir().unwrap();
        let baseline = root.path().join("baseline.png");
        let current = root.path().join("current.png");
        RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 255]))
            .save(&baseline)
            .unwrap();
        let mut changed = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 255]));
        changed.put_pixel(1, 1, Rgba([255, 255, 255, 255]));
        changed.save(&current).unwrap();

        assert_eq!(assert_png_changed(&baseline, &current, 1).unwrap(), 1);
        assert!(assert_png_changed(&baseline, &current, 2).is_err());
    }

    #[test]
    fn screenshot_rgb_assertion_counts_only_pixels_within_tolerance() {
        let root = tempfile::tempdir().unwrap();
        let screenshot = root.path().join("state.png");
        let mut image = RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]));
        image.put_pixel(1, 1, Rgba([12, 18, 31, 255]));
        image.save(&screenshot).unwrap();

        assert_eq!(
            assert_png_contains_rgb(&screenshot, [10, 20, 30], 2, 4).unwrap(),
            4
        );
        assert!(assert_png_contains_rgb(&screenshot, [10, 20, 30], 0, 4).is_err());
    }

    #[test]
    fn in_memory_color_region_locator_matches_the_png_locator() {
        let mut image = RgbaImage::from_pixel(8, 6, Rgba([0, 0, 0, 255]));
        for y in 1..4 {
            for x in 2..6 {
                image.put_pixel(x, y, Rgba([31, 95, 139, 255]));
            }
        }

        assert_eq!(
            locate_largest_color_region_in_image(&image, [31, 95, 139], 0, 4, None).unwrap(),
            super::ColorRegion {
                min_x: 2,
                max_x: 5,
                min_y: 1,
                max_y: 3,
                center_x: 3,
                center_y: 2,
                pixels: 12,
            }
        );
    }

    #[test]
    fn color_region_locator_limits_search_to_requested_bounds() {
        let mut image = RgbaImage::from_pixel(10, 8, Rgba([0, 0, 0, 255]));
        for y in 0..5 {
            for x in 0..5 {
                image.put_pixel(x, y, Rgba([31, 95, 139, 255]));
            }
        }
        for y in 5..8 {
            for x in 7..10 {
                image.put_pixel(x, y, Rgba([31, 95, 139, 255]));
            }
        }

        assert_eq!(
            locate_largest_color_region_in_image(
                &image,
                [31, 95, 139],
                0,
                4,
                Some(PngBounds {
                    x: 6,
                    y: 4,
                    width: 4,
                    height: 4,
                }),
            )
            .unwrap(),
            super::ColorRegion {
                min_x: 7,
                max_x: 9,
                min_y: 5,
                max_y: 7,
                center_x: 8,
                center_y: 6,
                pixels: 9,
            }
        );
    }

    #[test]
    fn color_region_locator_rejects_invalid_search_bounds() {
        let image = RgbaImage::from_pixel(8, 6, Rgba([31, 95, 139, 255]));

        for bounds in [
            PngBounds {
                x: 0,
                y: 0,
                width: 0,
                height: 1,
            },
            PngBounds {
                x: 7,
                y: 5,
                width: 2,
                height: 1,
            },
            PngBounds {
                x: u32::MAX,
                y: 0,
                width: 1,
                height: 1,
            },
        ] {
            assert!(locate_largest_color_region_in_image(
                &image,
                [31, 95, 139],
                0,
                1,
                Some(bounds)
            )
            .is_err());
        }
    }
}
