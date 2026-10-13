use katana_core::markdown::DiagramThemeSnapshot;
use katana_platform::theme::{Rgb, ThemeColors, ThemeMode};

const HEADER_ALPHA: f32 = 0.3;
const STRIPE_ALPHA: f32 = 0.1;

pub(super) struct ExpectedTheme;

impl ExpectedTheme {
    pub(super) fn assert_snapshot(colors: &ThemeColors, snapshot: &DiagramThemeSnapshot) {
        let preview = &colors.preview;
        assert_eq!(snapshot.name, colors.name);
        assert_eq!(snapshot.is_dark, colors.mode == ThemeMode::Dark);
        assert_eq!(snapshot.background, Self::hex(preview.background));
        assert_eq!(snapshot.text, Self::hex(preview.text));
        assert_eq!(snapshot.preview_text, Self::hex(preview.text));
        assert_eq!(snapshot.table_border, Some(Self::hex(preview.border)));
        assert_eq!(
            snapshot.table_header_background,
            Some(Self::blend(colors, HEADER_ALPHA))
        );
        assert_eq!(
            snapshot.table_even_row_background,
            Some(Self::blend(colors, STRIPE_ALPHA))
        );
    }

    fn hex(color: Rgb) -> String {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    }

    fn blend(colors: &ThemeColors, alpha: f32) -> String {
        let background = colors.preview.background;
        let selection = colors.preview.selection;
        let background = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(background);
        let selection = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(selection);
        let result = background.blend(selection.gamma_multiply(alpha));
        format!("#{:02x}{:02x}{:02x}", result.r(), result.g(), result.b())
    }
}
