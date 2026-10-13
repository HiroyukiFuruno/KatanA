use image::RgbImage;
use katana_core::markdown::DiagramThemeSnapshot;

mod contrast;
mod table;
use contrast::PdfContrast;
use table::TableThemeColors;

const COLOR_TOLERANCE: u8 = 12;
const MIN_BACKGROUND_PIXELS: usize = 2_000;
const MIN_TEXT_PIXELS: usize = 20;
const MIN_TABLE_COLOR_RUN_PIXELS: usize = 32;
const MIN_DIAGRAM_PIXELS: usize = 24;
const MINIMUM_TEXT_CONTRAST: f64 = 4.5;
const DIAGRAM_ACCENT: &str = "#D22B5F";
const RGB_CHANNELS: usize = 3;

#[derive(Clone, Copy)]
struct Rgb(u8, u8, u8);

pub(super) struct PdfThemeAssertions;

impl PdfThemeAssertions {
    pub(super) fn assert_theme_pixels(image: &RgbImage, theme: &DiagramThemeSnapshot, label: &str) {
        let background = Rgb::from_hex(&theme.background);
        let text = Rgb::from_hex(&theme.text);
        let rendered_background = mean_near_color(
            image,
            background,
            COLOR_TOLERANCE,
            MIN_BACKGROUND_PIXELS,
            label,
        );
        let rendered_text = mean_near_color(image, text, COLOR_TOLERANCE, MIN_TEXT_PIXELS, label);
        let contrast = PdfContrast::ratio(rendered_text, rendered_background);
        assert!(
            contrast >= MINIMUM_TEXT_CONTRAST,
            "{label}: rasterized PDF text contrast {contrast:.2}:1 is below {MINIMUM_TEXT_CONTRAST}:1"
        );
        assert_table_pixels(image, theme, label);
        let accent = Rgb::from_hex(DIAGRAM_ACCENT);
        let diagram_pixels = image
            .pixels()
            .filter(|pixel| near(pixel.0, accent, COLOR_TOLERANCE))
            .count();
        assert!(
            diagram_pixels >= MIN_DIAGRAM_PIXELS,
            "{label}: diagram accent color appears in only {diagram_pixels} raster pixels"
        );
    }

    pub(super) fn assert_table_colors_reject_background_substitution(
        image: &RgbImage,
        theme: &DiagramThemeSnapshot,
        label: &str,
    ) {
        let background = Rgb::from_hex(&theme.background);
        for (hex, minimum, role) in TableThemeColors::entries(theme) {
            let expected = Rgb::from_hex(hex);
            let tolerance = distinguishable_tolerance(expected, background).unwrap_or_else(|| {
                panic!("{label}: {role} matches page background; semantic ROI is required")
            });
            let mut substituted = image.clone();
            replace_near_color(&mut substituted, expected, background, tolerance);
            assert!(
                !has_color_region(&substituted, expected, minimum, background),
                "{label}: {role} verifier accepted pixels replaced with page background"
            );
        }
    }
}

fn assert_table_pixels(image: &RgbImage, theme: &DiagramThemeSnapshot, label: &str) {
    let background = Rgb::from_hex(&theme.background);
    for (hex, minimum, role) in TableThemeColors::entries(theme) {
        let expected = Rgb::from_hex(hex);
        assert!(
            distinguishable_tolerance(expected, background).is_some()
                && has_color_region(image, expected, minimum, background),
            "{label}: {role} has no distinct continuous region; background-identical tokens need semantic ROI"
        );
    }
}

fn has_color_region(image: &RgbImage, expected: Rgb, minimum: usize, background: Rgb) -> bool {
    let Some(tolerance) = distinguishable_tolerance(expected, background) else {
        return false;
    };
    let count = image
        .pixels()
        .filter(|pixel| near(pixel.0, expected, tolerance))
        .count();
    if count < minimum {
        return false;
    }
    (0..image.height()).fold(0, |longest, y| {
        let (run, _) = (0..image.width()).fold((0, 0), |(best, run), x| {
            let run = if near(image.get_pixel(x, y).0, expected, tolerance) {
                run + 1
            } else {
                0
            };
            (best.max(run), run)
        });
        longest.max(run)
    }) >= MIN_TABLE_COLOR_RUN_PIXELS
}
fn distinguishable_tolerance(expected: Rgb, background: Rgb) -> Option<u8> {
    let distance = expected
        .0
        .abs_diff(background.0)
        .max(expected.1.abs_diff(background.1))
        .max(expected.2.abs_diff(background.2));
    distance
        .checked_sub(1)
        .map(|separation| COLOR_TOLERANCE.min(separation / 2))
}

#[test]
fn background_identical_table_color_requires_semantic_roi_without_panicking() {
    let background = Rgb(16, 24, 32);
    assert_eq!(distinguishable_tolerance(background, background), None);
}
fn replace_near_color(image: &mut RgbImage, expected: Rgb, replacement: Rgb, tolerance: u8) {
    for pixel in image.pixels_mut() {
        if near(pixel.0, expected, tolerance) {
            *pixel = image::Rgb([replacement.0, replacement.1, replacement.2]);
        }
    }
}

fn mean_near_color(
    image: &RgbImage,
    expected: Rgb,
    tolerance: u8,
    minimum: usize,
    label: &str,
) -> Rgb {
    let (sums, count) = image
        .pixels()
        .filter(|pixel| near(pixel.0, expected, tolerance))
        .fold(
            ([0_u64; RGB_CHANNELS], 0_usize),
            |(mut sums, count), pixel| {
                for (sum, channel) in sums.iter_mut().zip(pixel.0) {
                    *sum += u64::from(channel);
                }
                (sums, count + 1)
            },
        );
    assert!(
        count >= minimum,
        "{label}: expected color has only {count} matching raster pixels"
    );
    Rgb(
        (sums[0] / count as u64) as u8,
        (sums[1] / count as u64) as u8,
        (sums[2] / count as u64) as u8,
    )
}

fn near(actual: [u8; RGB_CHANNELS], expected: Rgb, tolerance: u8) -> bool {
    actual
        .into_iter()
        .zip([expected.0, expected.1, expected.2])
        .all(|(a, e)| a.abs_diff(e) <= tolerance)
}

impl Rgb {
    fn from_hex(value: &str) -> Self {
        let (red, green, blue) =
            katana_core::markdown::color_preset::DiagramColorPreset::parse_hex_rgb(value)
                .unwrap_or_else(|| panic!("expected #RRGGBB theme token, got {value}"));
        Self(red, green, blue)
    }
}
