use super::{Icon, IconRegistry, IconSize, pack};

impl IconSize {
    pub const SMALL: f32 = 12.0;
    pub const MEDIUM: f32 = 16.0;
    pub const LARGE: f32 = 20.0;

    pub const fn to_vec2(self) -> egui::Vec2 {
        match self {
            Self::Small => egui::vec2(Self::SMALL, Self::SMALL),
            Self::Medium => egui::vec2(Self::MEDIUM, Self::MEDIUM),
            Self::Large => egui::vec2(Self::LARGE, Self::LARGE),
        }
    }
}

impl Icon {
    /* WHY: Returns the absolute byte-URI for the icon SVG. */
    pub fn uri(&self) -> String {
        format!("bytes://icon/{}.svg", self.name())
    }

    /* WHY: Returns an egui::Image for the icon at the given size. */
    pub fn image(&self, size: IconSize) -> egui::Image<'static> {
        egui::Image::new(self.uri())
            .fit_to_exact_size(size.to_vec2())
            .maintain_aspect_ratio(false)
    }

    /* WHY: Returns the default color for a specific vendor's icon pack. */
    pub fn vendor_default_color(&self, vendor: &str, is_dark: bool) -> Option<egui::Color32> {
        /* WHY: Do not apply colorful defaults to Katana itself by default, only third-party vendors. */
        if vendor == "katana" || vendor.is_empty() {
            return None;
        }

        /* WHY: Provide semantic colors for specific functional icons. */
        let hex = match self {
            /* WHY: Destructive / Error / Remove */
            Self::Error | Self::Remove | Self::Minus | Self::Close | Self::CloseModal => {
                if is_dark { "#FF6B6B" } else { "#DC3545" }
            }
            /* WHY: Warning / Folders */
            Self::Warning | Self::FolderOpen | Self::FolderClosed => {
                if is_dark {
                    "#FFD166"
                } else {
                    "#FFC107"
                }
            }
            /* WHY: Success / Add */
            Self::Success | Self::Plus => {
                if is_dark {
                    "#06D6A0"
                } else {
                    "#198754"
                }
            }
            /* WHY: Info / Systems / Files */
            Self::Info | Self::Document | Self::Markdown | Self::Explorer => {
                if is_dark {
                    "#118AB2"
                } else {
                    "#0D6EFD"
                }
            }
            /* WHY: Default fallback for vendor icons: Let them remain monochrome */
            _ => return None,
        };

        egui::Color32::from_hex(hex).ok()
    }

    /* WHY: Returns a theme-aware and potentially tinted egui::Image for the icon. */
    pub fn ui_image(&self, ui: &egui::Ui, size: IconSize) -> egui::Image<'static> {
        let image = self.image(size);
        if IconRegistry::get_render_policy(ui.ctx()) == pack::RenderPolicy::TintedMonochrome {
            let overrides = IconRegistry::get_active_overrides(ui.ctx());
            let icon_ov = overrides.as_ref().and_then(|o| o.0.get(self.name()));
            let mut base_color = ui.visuals().text_color();

            if IconRegistry::is_colorful_vendor_icons(ui.ctx()) {
                /* WHY: Allow individual color overrides first (Only if colorful policy is enabled) */
                if let Some(rgba) = icon_ov.and_then(|ov| ov.color) {
                    return image.tint(crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(rgba));
                }

                /* WHY: Fallback to colorful vendor checking */
                let default_pack = IconRegistry::get_default_pack_id(ui.ctx());
                let vendor = icon_ov
                    .and_then(|ov| ov.vendor.as_deref())
                    .unwrap_or_else(|| &default_pack);

                if let Some(c) = self.vendor_default_color(vendor, ui.visuals().dark_mode) {
                    base_color = c;
                }
            }
            image.tint(base_color)
        } else {
            image
        }
    }

    /* WHY: Returns an egui::Button with the icon and canonical background fill. */
    pub fn button(&self, ui: &egui::Ui, size: IconSize) -> egui::Button<'static> {
        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg()
        };
        egui::Button::image(self.ui_image(ui, size)).fill(icon_bg)
    }

    /* WHY: Like button, but applies selection_bg fill when selected is true. */
    pub fn selected_button(
        &self,
        ui: &egui::Ui,
        size: IconSize,
        selected: bool,
    ) -> egui::Button<'static> {
        let btn = self.button(ui, size);
        if selected {
            btn.fill(ui.visuals().selection.bg_fill)
        } else {
            btn
        }
    }

    /* WHY: Attempts to resolve a Katana icon from a standard emoji character. */
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
            '✨' => Some(Self::Action),
            _ => None,
        }
    }

    /* WHY: Returns the canonical emoji representation for the icon. */
    pub fn as_char(&self) -> char {
        match self {
            Self::Plus => '➕',
            Self::Document => '📄',
            Self::Markdown => '📝',
            Self::Recent | Self::History => '🕒',
            Self::Action => '⚡',
            Self::Refresh => '🔄',
            Self::Pin => '📌',
            Self::Warning => '⚠',
            Self::Rocket => '🚀',
            Self::Download => '⬇',
            Self::Hourglass => '⏳',
            Self::Info => 'ℹ',
            Self::LightBulb => '💡',
            _ => ' ',
        }
    }
}
