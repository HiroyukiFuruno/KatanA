use eframe::egui;

const RGBA_CHANNELS: usize = 4;
const ALPHA_CHANNEL_INDEX: usize = 3;

pub(crate) struct ImageBackgroundOps;

impl ImageBackgroundOps {
    pub(crate) fn paint(ui: &mut egui::Ui, rect: egui::Rect) {
        ui.painter().rect_filled(
            rect,
            0.0,
            Self::preview_background(ui.ctx(), ui.visuals().window_fill()),
        );
    }

    pub(super) fn preview_background(
        ctx: &egui::Context,
        fallback: egui::Color32,
    ) -> egui::Color32 {
        ctx.data(|data| {
            data.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                "katana_theme_colors",
            ))
        })
        .map_or_else(
            || fallback,
            |theme| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(theme.preview.background),
        )
    }

    pub(super) fn composite_rgba_over_background(rgba: &mut [u8], background: egui::Color32) {
        for pixel in rgba.chunks_exact_mut(RGBA_CHANNELS) {
            let alpha = pixel[ALPHA_CHANNEL_INDEX];
            if alpha == u8::MAX {
                continue;
            }
            pixel[0] = composite_channel(pixel[0], alpha, background.r());
            pixel[1] = composite_channel(pixel[1], alpha, background.g());
            pixel[2] = composite_channel(pixel[2], alpha, background.b());
            pixel[ALPHA_CHANNEL_INDEX] = u8::MAX;
        }
    }
}

fn composite_channel(foreground: u8, alpha: u8, background: u8) -> u8 {
    let foreground = u16::from(foreground) * u16::from(alpha);
    let background = u16::from(background) * u16::from(u8::MAX - alpha);
    ((foreground + background) / u16::from(u8::MAX)) as u8
}

#[cfg(test)]
mod tests {
    use super::ImageBackgroundOps;
    use eframe::egui;

    #[test]
    fn preview_background_uses_theme_preview_background() {
        let ctx = egui::Context::default();
        let theme = katana_platform::theme::ThemePreset::KatanaLight.colors();
        ctx.data_mut(|data| {
            data.insert_temp(egui::Id::new("katana_theme_colors"), theme.clone());
        });

        assert_eq!(
            ImageBackgroundOps::preview_background(&ctx, egui::Color32::BLACK),
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(theme.preview.background)
        );
    }

    #[test]
    fn composite_rgba_replaces_transparent_pixels_with_background() {
        let mut rgba = vec![255, 0, 0, 0, 10, 20, 30, 255];
        let background = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
            katana_platform::theme::ThemePreset::SolarizedLight
                .colors()
                .preview
                .background,
        );

        ImageBackgroundOps::composite_rgba_over_background(&mut rgba, background);

        assert_eq!(
            rgba,
            vec![
                background.r(),
                background.g(),
                background.b(),
                255,
                10,
                20,
                30,
                255
            ]
        );
    }
}
