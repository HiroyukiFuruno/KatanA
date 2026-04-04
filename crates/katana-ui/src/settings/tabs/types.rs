use katana_platform::theme::{Rgb, Rgba, ThemeColors};

pub struct BehaviorTabOps;
pub struct FontTabOps;
pub struct LayoutTabOps;
pub struct ThemeTabOps;
pub struct UpdatesTabOps;
pub struct WorkspaceTabOps;

pub(crate) enum ColorPropType {
    Rgb(fn(&ThemeColors) -> Rgb, fn(&mut ThemeColors, Rgb)),
    Rgba(fn(&ThemeColors) -> Rgba, fn(&mut ThemeColors, Rgba)),
}
