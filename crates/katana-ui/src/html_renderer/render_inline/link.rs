/* WHY: Encapsulated link rendering logic to manage UI complexity and maintain strict line limits. */

use super::super::EMOJI_INLINE_PIXEL_SIZE;
use super::super::types::HtmlRenderer;
use super::text::HtmlInlineTextOps;
use eframe::egui;
use katana_core::emoji::EmojiRasterOps;
use katana_core::html::{HtmlNode, LinkAction};
use unicode_segmentation::UnicodeSegmentation;

pub struct HtmlInlineLinkOps;

impl HtmlInlineLinkOps {
    pub(crate) fn render_link_with_images<'a>(
        renderer: &mut HtmlRenderer<'a>,
        children: &[HtmlNode],
        action: &LinkAction,
        tooltip: &str,
    ) -> Option<LinkAction> {
        let mut clicked = false;
        for child in children {
            if let HtmlNode::Image { src, alt: _ } = child {
                let url = super::super::ensure_svg_extension(src);
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
                Self::render_text_node_in_link(renderer, t, tooltip, &mut clicked);
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
                HtmlInlineTextOps::render_emoji_with_underline(
                    ui, grapheme, bytes, tooltip, clicked,
                );
            }
            flush_text(ui, &mut text_buffer, clicked);
        });
    }

    pub(crate) fn render_link_text<'a>(
        renderer: &mut HtmlRenderer<'a>,
        text: &str,
        action: LinkAction,
        tooltip: &str,
    ) -> Option<LinkAction> {
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
                HtmlInlineTextOps::render_emoji_with_underline(
                    ui,
                    grapheme,
                    bytes,
                    tooltip,
                    &mut clicked,
                );
            }
            flush_text(ui, &mut text_buffer, &mut clicked);
        });

        if clicked { Some(action) } else { None }
    }
}
