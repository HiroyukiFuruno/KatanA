/* WHY: Refactored HTML inline renderer entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use eframe::egui;
use katana_core::emoji::EmojiRasterOps;
use katana_core::html::{HtmlNode, LinkAction};
use unicode_segmentation::UnicodeSegmentation;

use super::types::HtmlRenderer;
use super::{EMOJI_INLINE_DISPLAY_SIZE, EMOJI_INLINE_PIXEL_SIZE, LINE_BREAK_SPACING};

mod link;
mod text;

impl<'a> HtmlRenderer<'a> {
    pub(super) fn render_inline(&mut self, node: &HtmlNode) -> Option<LinkAction> {
        match node {
            HtmlNode::Text(text) => {
                let mut text_buffer = String::new();
                let flush_text =
                    |ui: &mut egui::Ui,
                     text_buffer: &mut String,
                     is_strong: bool,
                     is_italics: bool,
                     text_color: Option<egui::Color32>| {
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
                        if let Some(c) = text_color {
                            rt = rt.color(c);
                        }
                        ui.label(rt);
                        text_buffer.clear();
                    };

                let is_strong = self.is_strong;
                let is_italics = self.is_italics;
                let text_color = self.text_color;

                self.ui.scope(|ui| {
                    for grapheme in text.graphemes(true) {
                        if let Some(bytes) = EmojiRasterOps::render_apple_color_emoji_png(
                            grapheme,
                            EMOJI_INLINE_PIXEL_SIZE,
                        ) {
                            flush_text(ui, &mut text_buffer, is_strong, is_italics, text_color);
                            let uri = format!("emoji://{grapheme}");
                            ui.add(egui::Image::from_bytes(uri, bytes).fit_to_exact_size(
                                egui::vec2(EMOJI_INLINE_DISPLAY_SIZE, EMOJI_INLINE_DISPLAY_SIZE),
                            ));
                        } else {
                            text_buffer.push_str(grapheme);
                        }
                    }
                    flush_text(ui, &mut text_buffer, is_strong, is_italics, text_color);
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
                    link::HtmlInlineLinkOps::render_link_with_images(
                        self, children, &action, &tooltip,
                    )
                } else {
                    link::HtmlInlineLinkOps::render_link_text(self, &text, action, &tooltip)
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
