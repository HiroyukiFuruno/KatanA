use super::{RGB_CHANNELS, Rgb};

const LUMINANCE_OFFSET: f64 = 0.05;
const MAX_SRGB_CHANNEL: f64 = 255.0;
const SRGB_LINEAR_BOUNDARY: f64 = 0.04045;
const SRGB_LINEAR_DIVISOR: f64 = 12.92;
const SRGB_NONLINEAR_OFFSET: f64 = 0.055;
const SRGB_NONLINEAR_SCALE: f64 = 1.055;
const SRGB_GAMMA: f64 = 2.4;
const CHANNEL_WEIGHTS: [f64; RGB_CHANNELS] = [0.2126, 0.7152, 0.0722];

pub(super) struct PdfContrast;

impl PdfContrast {
    pub(super) fn ratio(first: Rgb, second: Rgb) -> f64 {
        let (first, second) = (Self::luminance(first), Self::luminance(second));
        (first.max(second) + LUMINANCE_OFFSET) / (first.min(second) + LUMINANCE_OFFSET)
    }

    fn luminance(color: Rgb) -> f64 {
        [color.0, color.1, color.2]
            .into_iter()
            .zip(CHANNEL_WEIGHTS)
            .map(|(channel, weight)| {
                let srgb = f64::from(channel) / MAX_SRGB_CHANNEL;
                weight
                    * if srgb <= SRGB_LINEAR_BOUNDARY {
                        srgb / SRGB_LINEAR_DIVISOR
                    } else {
                        ((srgb + SRGB_NONLINEAR_OFFSET) / SRGB_NONLINEAR_SCALE).powf(SRGB_GAMMA)
                    }
            })
            .sum()
    }
}
