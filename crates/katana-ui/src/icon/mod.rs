pub mod pack;
mod types;
pub use types::{ALL_ICONS, Icon, IconSize};

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

pub use types::IconOps;

impl IconOps {
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
            let mut acc = |r: egui::Response| {
                response = Some(match response.take() {
                    Some(mut e) => {
                        e |= r;
                        e
                    }
                    None => r,
                });
            };

            for (idx, ch) in text.char_indices() {
                let Some(icon) = Icon::try_from_emoji(ch) else {
                    continue;
                };
                if last_end < idx {
                    acc(ui.label(egui::RichText::new(&text[last_end..idx]).color(text_color)));
                }
                let image = icon.image(IconSize::Medium);
                let image = if IconRegistry::get_render_policy(ui.ctx())
                    == pack::RenderPolicy::TintedMonochrome
                {
                    image.tint(text_color)
                } else {
                    image
                };
                acc(ui.add(image));
                last_end = idx + ch.len_utf8();
            }

            if last_end < text.len() {
                acc(ui.label(egui::RichText::new(&text[last_end..]).color(text_color)));
            }

            response.unwrap_or_else(|| ui.label(""))
        })
        .inner
    }

    pub fn button_with_icon_str(ui: &mut egui::Ui, text: &str) -> egui::Response {
        let mut chars = text.chars();
        if let Some(first_char) = chars.next()
            && let Some(icon) = Icon::try_from_emoji(first_char)
        {
            let rest = chars.as_str().trim_start();
            return ui.add(egui::Button::image_and_text(
                icon.ui_image(ui, IconSize::Small),
                rest,
            ));
        }
        ui.button(text)
    }
}

pub use types::IconRegistry;

#[derive(Clone, Copy)]
pub struct ActiveRenderPolicy(pub pack::RenderPolicy);

impl IconRegistry {
    pub fn install(ctx: &egui::Context) {
        Self::install_pack(ctx, &pack::KatanaIconPack);
    }

    pub fn install_pack_by_id(ctx: &egui::Context, _pack_id: &str) {
        Self::install_pack(ctx, &pack::KatanaIconPack);
    }

    pub fn install_pack(ctx: &egui::Context, pack: &dyn pack::IconPackContract) {
        let fallback_pack = pack::KatanaIconPack;

        for icon in ALL_ICONS {
            let bytes = pack
                .get_asset(*icon)
                .or_else(|| pack::IconPackContract::get_asset(&fallback_pack, *icon))
                .expect("KatanaIconPack must provide all icons");

            ctx.include_bytes(icon.uri(), bytes);
        }

        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("katana_icon_render_policy"),
                ActiveRenderPolicy(pack.manifest().render_policy),
            )
        });
        ctx.forget_all_images();
    }

    pub fn get_render_policy(ctx: &egui::Context) -> pack::RenderPolicy {
        ctx.data(|d| {
            d.get_temp::<ActiveRenderPolicy>(egui::Id::new("katana_icon_render_policy"))
                .map(|p| p.0)
                .unwrap_or(pack::RenderPolicy::TintedMonochrome)
        })
    }
}

#[cfg(test)]
mod tests;
