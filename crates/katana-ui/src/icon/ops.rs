use super::{Icon, IconRegistry, IconSize, pack, types::IconOps};

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
                    let mut color = text_color;
                    if IconRegistry::is_colorful_vendor_icons(ui.ctx()) {
                        let vendor = IconRegistry::get_default_pack_id(ui.ctx());
                        color = icon
                            .vendor_default_color(&vendor, ui.visuals().dark_mode)
                            .unwrap_or(color);
                    }
                    image.tint(color)
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
