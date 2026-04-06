use eframe::egui;
use katana_core::html::{HtmlNode, LinkAction};

use super::types::HtmlRenderer;
use super::{
    EMOJI_INLINE_DISPLAY_SIZE, EMOJI_INLINE_NEGATIVE_SPACE, EMOJI_INLINE_PIXEL_SIZE,
    EMOJI_INLINE_UNDERLINE_OFFSET_Y, LINE_BREAK_SPACING,
};

impl<'a> HtmlRenderer<'a> {
    pub(super) fn render_inline(&mut self, node: &HtmlNode) -> Option<LinkAction> {
        match node {
            HtmlNode::Text(text) => {
                use katana_core::emoji::EmojiRasterOps;
                use unicode_segmentation::UnicodeSegmentation;

                let mut text_buffer = String::new();
                let flush_text = |ui: &mut egui::Ui, text_buffer: &mut String| {
                    if text_buffer.is_empty() {
                        return;
                    }
                    let mut rt = egui::RichText::new(&*text_buffer);
                    if self.is_strong {
                        rt = rt.strong();
                    }
                    if self.is_italics {
                        rt = rt.italics();
                    }
                    if let Some(c) = self.text_color {
                        rt = rt.color(c);
                    }
                    ui.label(rt);
                    text_buffer.clear();
                };

                self.ui.scope(|ui| {
                    for grapheme in text.graphemes(true) {
                        if let Some(bytes) = EmojiRasterOps::render_apple_color_emoji_png(
                            grapheme,
                            EMOJI_INLINE_PIXEL_SIZE,
                        ) {
                            flush_text(ui, &mut text_buffer);
                            let uri = format!("emoji://{grapheme}");
                            ui.add(egui::Image::from_bytes(uri, bytes).fit_to_exact_size(
                                egui::vec2(EMOJI_INLINE_DISPLAY_SIZE, EMOJI_INLINE_DISPLAY_SIZE),
                            ));
                        } else {
                            text_buffer.push_str(grapheme);
                        }
                    }
                    flush_text(ui, &mut text_buffer);
                });
                None
            }
            HtmlNode::Image { src, alt: _ } => {
                let url = super::ensure_svg_extension(src);
                self.ui.add(
                    egui::Image::new(url)
                        .fit_to_original_size(1.0)
                        .max_width(self.max_image_width),
                );
                None
            }
            HtmlNode::Link { target, children } => {
                let text = super::collect_text(children);
                let action = target.default_action();
                let tooltip = target.tooltip_text();
                let has_images = children.iter().any(|c| matches!(c, HtmlNode::Image { .. }));

                if has_images {
                    render_link_with_images(self, children, &action, &tooltip)
                } else {
                    render_link_text(self, &text, action, &tooltip)
                }
            }
            HtmlNode::LineBreak => {
                self.ui.add_space(LINE_BREAK_SPACING);
                None
            }
            HtmlNode::Emphasis(children) => {
                let prev = self.is_italics;
                self.is_italics = true;
                let mut action = None;
                for child in children {
                    if let Some(a) = self.render_inline(child) {
                        action = Some(a);
                    }
                }
                self.is_italics = prev;
                action
            }
            HtmlNode::Strong(children) => {
                let prev = self.is_strong;
                self.is_strong = true;
                let mut action = None;
                for child in children {
                    if let Some(a) = self.render_inline(child) {
                        action = Some(a);
                    }
                }
                self.is_strong = prev;
                action
            }
            _ => None,
        }
    }
}

fn render_link_with_images<'a>(
    renderer: &mut HtmlRenderer<'a>,
    children: &[HtmlNode],
    action: &LinkAction,
    tooltip: &str,
) -> Option<LinkAction> {
    let mut clicked = false;
    for child in children {
        if let HtmlNode::Image { src, alt: _ } = child {
            let url = super::ensure_svg_extension(src);
            let response = renderer.ui.add(
                egui::Image::new(url)
                    .fit_to_original_size(1.0)
                    .max_width(renderer.max_image_width)
                    .sense(egui::Sense::click()),
            );
            let response = response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(tooltip);
            if response.clicked() {
                clicked = true;
            }
        } else if let HtmlNode::Text(t) = child {
            render_text_node_in_link(renderer, t, tooltip, &mut clicked);
        }
    }
    if clicked { Some(action.clone()) } else { None }
}

fn render_text_node_in_link(
    renderer: &mut HtmlRenderer<'_>,
    t: &str,
    tooltip: &str,
    clicked: &mut bool,
) {
    use katana_core::emoji::EmojiRasterOps;
    use unicode_segmentation::UnicodeSegmentation;
    let is_strong = renderer.is_strong;
    let is_italics = renderer.is_italics;
    let mut text_buffer = String::new();
    let flush_text = |ui: &mut egui::Ui, text_buffer: &mut String, clicked: &mut bool| {
        if text_buffer.is_empty() {
            return;
        }
        let mut rt = egui::RichText::new(&*text_buffer);
        if is_strong {
            rt = rt.strong();
        }
        if is_italics {
            rt = rt.italics();
        }
        rt = rt.color(ui.visuals().hyperlink_color).underline();
        let response = ui
            .add(egui::Label::new(rt).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tooltip);
        if response.clicked() {
            *clicked = true;
        }
        text_buffer.clear();
    };
    renderer.ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for grapheme in t.graphemes(true) {
            let Some(bytes) =
                EmojiRasterOps::render_apple_color_emoji_png(grapheme, EMOJI_INLINE_PIXEL_SIZE)
            else {
                text_buffer.push_str(grapheme);
                continue;
            };
            flush_text(ui, &mut text_buffer, clicked);
            render_emoji_with_underline(ui, grapheme, bytes, tooltip, clicked);
        }
        flush_text(ui, &mut text_buffer, clicked);
    });
}

fn render_emoji_with_underline(
    ui: &mut egui::Ui,
    grapheme: &str,
    bytes: Vec<u8>,
    tooltip: &str,
    clicked: &mut bool,
) {
    let uri = format!("emoji://{grapheme}");
    let mut response = ui.add(
        egui::Image::from_bytes(uri, bytes)
            .fit_to_exact_size(egui::vec2(
                EMOJI_INLINE_DISPLAY_SIZE,
                EMOJI_INLINE_DISPLAY_SIZE,
            ))
            .sense(egui::Sense::click()),
    );
    response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip);
    let y = response.rect.max.y + EMOJI_INLINE_UNDERLINE_OFFSET_Y;
    ui.painter().hline(
        response.rect.x_range(),
        y,
        egui::Stroke::new(1.0, ui.visuals().hyperlink_color),
    );
    if response.clicked() {
        *clicked = true;
    }
    ui.add_space(EMOJI_INLINE_NEGATIVE_SPACE);
}

fn render_link_text<'a>(
    renderer: &mut HtmlRenderer<'a>,
    text: &str,
    action: LinkAction,
    tooltip: &str,
) -> Option<LinkAction> {
    use katana_core::emoji::EmojiRasterOps;
    use unicode_segmentation::UnicodeSegmentation;

    let mut clicked = false;
    let mut text_buffer = String::new();
    let is_strong = renderer.is_strong;
    let is_italics = renderer.is_italics;
    let flush_text = |ui: &mut egui::Ui, text_buffer: &mut String, clicked: &mut bool| {
        if text_buffer.is_empty() {
            return;
        }
        let mut rt = egui::RichText::new(&*text_buffer);
        if is_strong {
            rt = rt.strong();
        }
        if is_italics {
            rt = rt.italics();
        }
        rt = rt.color(ui.visuals().hyperlink_color).underline();
        let response = ui
            .add(egui::Label::new(rt).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tooltip);
        if response.clicked() {
            *clicked = true;
        }
        text_buffer.clear();
    };

    renderer.ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for grapheme in text.graphemes(true) {
            let Some(bytes) =
                EmojiRasterOps::render_apple_color_emoji_png(grapheme, EMOJI_INLINE_PIXEL_SIZE)
            else {
                text_buffer.push_str(grapheme);
                continue;
            };
            flush_text(ui, &mut text_buffer, &mut clicked);
            render_emoji_with_underline(ui, grapheme, bytes, tooltip, &mut clicked);
        }
        flush_text(ui, &mut text_buffer, &mut clicked);
    });

    if clicked { Some(action) } else { None }
}
