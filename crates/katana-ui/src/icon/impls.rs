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
    pub fn uri(&self) -> String {
        format!("bytes://icon/{}.svg", self.name())
    }

    pub fn image(&self, size: IconSize) -> egui::Image<'static> {
        egui::Image::new(self.uri())
            .fit_to_exact_size(size.to_vec2())
            .maintain_aspect_ratio(false)
    }

    pub fn ui_image(&self, ui: &egui::Ui, size: IconSize) -> egui::Image<'static> {
        let image = self.image(size);
        if IconRegistry::get_render_policy(ui.ctx()) == pack::RenderPolicy::TintedMonochrome {
            image.tint(ui.visuals().text_color())
        } else {
            image
        }
    }

    /// Returns an `egui::Button::image` with the canonical icon-bg fill applied.
    /// This is the only sanctioned call-site for `Button::image`; callers must use
    /// this method instead of constructing the button directly.
    pub fn button(&self, ui: &egui::Ui, size: IconSize) -> egui::Button<'static> {
        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg()
        };
        egui::Button::image(self.ui_image(ui, size)).fill(icon_bg)
    }

    /// Like `button`, but applies `selection_bg` fill when `selected` is true.
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
}
