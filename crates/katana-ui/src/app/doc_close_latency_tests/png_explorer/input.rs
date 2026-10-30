use super::{IMAGE_NAME, OFFICE_NAME};
use std::path::{Path, PathBuf};

const PNG_WIDTH: u32 = 2886;
const PNG_HEIGHT: u32 = 426;
const PNG_NOISE_HEIGHT: u32 = 6;
const PNG_RANDOM_SEED: u32 = 0x385_2026;
const HTML_PARAGRAPH_COUNT: usize = 1000;
const RANDOM_LEFT_SHIFT_A: u32 = 13;
const RANDOM_RIGHT_SHIFT: u32 = 17;
const RANDOM_LEFT_SHIFT_B: u32 = 5;
const PNG_BACKGROUND_GRAY: u8 = 224;
const PNG_MIN_BYTES: u64 = 80_000;
const PNG_MAX_BYTES: u64 = 180_000;
const INPUT_DIRECTORY_VARIABLE: &str = "KATANA_EXPLORER_LOADING_FIXTURE_DIR";

pub(super) struct LoadingFixtureInputs;

impl LoadingFixtureInputs {
    pub(super) fn office_path() -> PathBuf {
        Self::supplied_input(OFFICE_NAME).unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format")
                .join(OFFICE_NAME)
        })
    }

    fn supplied_input(name: &str) -> Option<PathBuf> {
        let directory = std::env::var_os(INPUT_DIRECTORY_VARIABLE)?;
        let path = PathBuf::from(directory).join(name);
        assert!(
            path.is_file(),
            "configured loading fixture is missing: {}",
            path.display()
        );
        Some(path)
    }

    pub(super) fn create_html(path: &Path) {
        if let Some(source) = Self::supplied_input("loading.html") {
            std::fs::copy(source, path).expect("copy configured HTML fixture");
            return;
        }
        let mut html = String::from("<!doctype html><html><body>");
        for index in 0..HTML_PARAGRAPH_COUNT {
            html.push_str(&format!(
                "<p>Local loading fixture paragraph {index:04}</p>"
            ));
        }
        html.push_str("</body></html>");
        std::fs::write(path, html).expect("write self-contained HTML loading fixture");
    }
    pub(super) fn create_png(path: &Path) {
        if let Some(source) = Self::supplied_input(IMAGE_NAME) {
            std::fs::copy(source, path).expect("copy configured PNG fixture");
            return;
        }
        let mut random = PNG_RANDOM_SEED;
        let image = image::RgbaImage::from_fn(PNG_WIDTH, PNG_HEIGHT, |_, y| {
            let gray = if y < PNG_NOISE_HEIGHT {
                random ^= random << RANDOM_LEFT_SHIFT_A;
                random ^= random >> RANDOM_RIGHT_SHIFT;
                random ^= random << RANDOM_LEFT_SHIFT_B;
                random as u8
            } else {
                PNG_BACKGROUND_GRAY
            };
            image::Rgba([gray, gray, gray, u8::MAX])
        });
        image
            .save(path)
            .expect("write generated PNG loading fixture");
        let bytes = std::fs::metadata(path).expect("stat generated PNG").len();
        assert!(
            (PNG_MIN_BYTES..PNG_MAX_BYTES).contains(&bytes),
            "generated PNG size {bytes} is outside the expected ~100KB range"
        );
    }
}
