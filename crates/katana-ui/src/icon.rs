macro_rules! define_icons {
    ( $( $(#[$meta:meta])* $variant:ident => $file:literal ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $( $(#[$meta])* $variant, )+
        }

        impl Icon {
            pub const fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $file, )+
                }
            }

            pub fn svg_bytes(&self) -> &'static [u8] {
                match self {
                    $( Self::$variant => include_bytes!(
                        concat!("../../../assets/icons/", $file, ".svg")
                    ), )+
                }
            }
        }

        pub const ALL_ICONS: &[Icon] = &[
            $( Icon::$variant, )+
        ];
    };
}

define_icons! {
    Dot             => "dot",
    ChevronLeft     => "chevron_left",
    ChevronRight    => "chevron_right",
    Refresh         => "refresh",
    Close           => "close",
    Remove          => "remove",
    ExternalLink    => "external_link",
    TriangleDown    => "triangle_down",
    TriangleLeft    => "triangle_left",
    TriangleRight   => "triangle_right",
    Search          => "search",
    Plus            => "plus",
    Minus           => "minus",
    Toc             => "toc",
    PanUp           => "pan_up",
    PanDown         => "pan_down",
    PanLeft         => "pan_left",
    PanRight        => "pan_right",
    ZoomIn          => "zoom_in",
    ZoomOut         => "zoom_out",
    ResetView       => "reset_view",
    Fullscreen      => "fullscreen",
    CloseModal      => "close_modal",
    Info            => "info",
    Success         => "success",
    Warning         => "warning",
    Error           => "error",
    Export          => "export",
    Filter          => "filter",
    Pin             => "pin",
    SplitVertical   => "split_vertical",
    SplitHorizontal => "split_horizontal",
    SwapHorizontal  => "swap_horizontal",
    SwapVertical    => "swap_vertical",
    Preview         => "preview",
    Document        => "document",
    FolderOpen      => "folder_open",
    FolderClosed    => "folder_closed",
    Copy            => "copy",
    ExpandAll       => "expand_all",
    CollapseAll     => "collapse_all",
    Github          => "github",
    Heart           => "heart",
    Bug             => "bug",
    Action          => "action",
    Recent          => "recent",
    Markdown        => "markdown",
    Rocket          => "rocket",
    Download        => "download",
    Hourglass       => "hourglass",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Small,
    Medium,
    Large,
}

impl IconSize {
    const SMALL: f32 = 12.0;
    const MEDIUM: f32 = 16.0;
    const LARGE: f32 = 20.0;

    pub const fn to_vec2(self) -> egui::Vec2 {
        match self {
            Self::Small => egui::vec2(Self::SMALL, Self::SMALL),
            Self::Medium => egui::vec2(Self::MEDIUM, Self::MEDIUM),
            Self::Large => egui::vec2(Self::LARGE, Self::LARGE),
        }
    }
}

impl Icon {
    pub fn uri(&self) -> String {
        format!("bytes://icon/{}.svg", self.name())
    }

    pub fn image(&self, size: IconSize) -> egui::Image<'static> {
        egui::Image::new(self.uri()).fit_to_exact_size(size.to_vec2())
    }

    pub fn ui_image(&self, ui: &egui::Ui, size: IconSize) -> egui::Image<'static> {
        self.image(size).tint(ui.visuals().text_color())
    }

    pub fn try_from_emoji(emoji: char) -> Option<Self> {
        match emoji {
            '📄' => Some(Self::Document),
            '📝' => Some(Self::Markdown),
            '🕒' => Some(Self::Recent),
            '⚡' => Some(Self::Action),
            '🔄' => Some(Self::Refresh),
            '📌' => Some(Self::Pin),
            '⚠' => Some(Self::Warning),
            '🚀' => Some(Self::Rocket),
            '⬇' => Some(Self::Download),
            '⏳' => Some(Self::Hourglass),
            '✨' => Some(Self::Action), // Wait, map Sparkles to Action for now, or create Sparkles
            _ => None,
        }
    }
}

pub fn render_str_with_icons(
    ui: &mut egui::Ui,
    text: &str,
    color: Option<egui::Color32>,
) -> egui::Response {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;

        let text_color = color.unwrap_or_else(|| ui.visuals().text_color());
        let mut last_end = 0;
        let mut response: Option<egui::Response> = None;

        for (idx, ch) in text.char_indices() {
            if let Some(icon) = Icon::try_from_emoji(ch) {
                if last_end < idx {
                    let chunk = &text[last_end..idx];
                    let r = ui.label(egui::RichText::new(chunk).color(text_color));
                    response = Some(if let Some(mut existing) = response {
                        existing |= r;
                        existing
                    } else {
                        r
                    });
                }

                let r = ui.add(icon.image(IconSize::Medium).tint(text_color));
                response = Some(if let Some(mut existing) = response {
                    existing |= r;
                    existing
                } else {
                    r
                });

                last_end = idx + ch.len_utf8();
            }
        }

        if last_end < text.len() {
            let r = ui.label(egui::RichText::new(&text[last_end..]).color(text_color));
            response = Some(if let Some(mut existing) = response {
                existing |= r;
                existing
            } else {
                r
            });
        }

        response.unwrap_or_else(|| ui.label(""))
    })
    .inner
}

pub fn button_with_icon_str(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let mut chars = text.chars();
    if let Some(first_char) = chars.next() {
        if let Some(icon) = Icon::try_from_emoji(first_char) {
            let rest = chars.as_str().trim_start();
            return ui.add(egui::Button::image_and_text(
                icon.ui_image(ui, IconSize::Small),
                rest,
            ));
        }
    }
    ui.button(text)
}

pub struct IconRegistry;

impl IconRegistry {
    pub fn install(ctx: &egui::Context) {
        for icon in ALL_ICONS {
            ctx.include_bytes(icon.uri(), icon.svg_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_size_to_vec2_returns_correct_dimensions() {
        assert_eq!(
            IconSize::Small.to_vec2(),
            egui::vec2(IconSize::SMALL, IconSize::SMALL)
        );
        assert_eq!(
            IconSize::Medium.to_vec2(),
            egui::vec2(IconSize::MEDIUM, IconSize::MEDIUM)
        );
        assert_eq!(
            IconSize::Large.to_vec2(),
            egui::vec2(IconSize::LARGE, IconSize::LARGE)
        );
    }

    #[test]
    fn icon_name_returns_snake_case_identifier() {
        assert_eq!(Icon::Refresh.name(), "refresh");
        assert_eq!(Icon::ChevronLeft.name(), "chevron_left");
        assert_eq!(Icon::ExternalLink.name(), "external_link");
        assert_eq!(Icon::ZoomIn.name(), "zoom_in");
    }

    #[test]
    fn icon_uri_follows_bytes_scheme() {
        assert_eq!(Icon::Refresh.uri(), "bytes://icon/refresh.svg");
        assert_eq!(Icon::ChevronLeft.uri(), "bytes://icon/chevron_left.svg");
    }

    #[test]
    fn all_icons_have_valid_svg_bytes() {
        for icon in ALL_ICONS {
            let bytes = icon.svg_bytes();
            let svg_str = std::str::from_utf8(bytes)
                .unwrap_or_else(|_| panic!("icon {:?} has invalid UTF-8", icon));
            assert!(
                svg_str.contains("<svg"),
                "icon {:?} SVG bytes must contain <svg tag",
                icon
            );
        }
    }

    #[test]
    fn all_icons_list_covers_every_variant() {
        assert_eq!(ALL_ICONS.len(), 50);
    }

    #[test]
    fn icon_registry_install_registers_all_icons() {
        let ctx = egui::Context::default();
        IconRegistry::install(&ctx);
    }

    #[test]
    fn icon_bytes_survive_forget_all_images_after_re_install() {
        let ctx = egui::Context::default();
        crate::svg_loader::install_image_loaders(&ctx);
        IconRegistry::install(&ctx);

        ctx.forget_all_images();

        IconRegistry::install(&ctx);

        let result = ctx.try_load_bytes(&Icon::Refresh.uri());
        assert!(
            result.is_ok(),
            "Icon bytes must be available after forget_all_images + re-install"
        );
    }

    #[test]
    fn try_from_emoji_maps_correctly() {
        assert_eq!(Icon::try_from_emoji('📄'), Some(Icon::Document));
        assert_eq!(Icon::try_from_emoji('📝'), Some(Icon::Markdown));
        assert_eq!(Icon::try_from_emoji('🕒'), Some(Icon::Recent));
        assert_eq!(Icon::try_from_emoji('⚡'), Some(Icon::Action));
        assert_eq!(Icon::try_from_emoji('🔄'), Some(Icon::Refresh));
        assert_eq!(Icon::try_from_emoji('📌'), Some(Icon::Pin));
        assert_eq!(Icon::try_from_emoji('⚠'), Some(Icon::Warning));
        assert_eq!(Icon::try_from_emoji('🚀'), Some(Icon::Rocket));
        assert_eq!(Icon::try_from_emoji('⬇'), Some(Icon::Download));
        assert_eq!(Icon::try_from_emoji('⏳'), Some(Icon::Hourglass));
        assert_eq!(Icon::try_from_emoji('✨'), Some(Icon::Action));
        assert_eq!(Icon::try_from_emoji('A'), None);
        assert_eq!(Icon::try_from_emoji('1'), None);
    }

    #[test]
    fn render_str_with_icons_renders_correctly() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Test string with emoji at beginning
                render_str_with_icons(ui, "⬇ Download", None);
                // Test string with emoji in middle
                render_str_with_icons(ui, "Wait ⏳ Please", None);
                // Test string with emoji at end
                render_str_with_icons(ui, "Done ✨", None);
                // Test string with multiple emojis
                render_str_with_icons(ui, "⬇ Download ⏳ Please ✨", None);
                // Test string without emojis
                render_str_with_icons(ui, "Normal text", None);
            });
        });
    }

    #[test]
    fn button_with_icon_str_renders_correctly() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Button with matching emoji
                let resp1 = button_with_icon_str(ui, "⬇ Install");
                assert!(resp1.rect.is_positive());

                // Button without matching emoji
                let resp2 = button_with_icon_str(ui, "Cancel");
                assert!(resp2.rect.is_positive());
            });
        });
    }
}
