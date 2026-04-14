use eframe::egui;
use katana_core::html::{HtmlNode, LinkAction};
use std::path::Path;

mod types;
pub use types::HtmlRenderer;
mod append_text;
mod block_render;
mod render_inline;

fn svg_badge_hosts() -> Vec<&'static str> {
    vec!["img.shields.io"]
}

const LINE_BREAK_SPACING: f32 = 4.0;
const HEADING_H2_SIZE: f32 = 20.0;
const HEADING_H3_SIZE: f32 = 16.0;
const PARAGRAPH_BLOCK_MARGIN_Y: f32 = 5.0;
const HEADING_BLOCK_MARGIN_Y: f32 = 6.0;
const EMOJI_INLINE_PIXEL_SIZE: u32 = 16;
const EMOJI_INLINE_DISPLAY_SIZE: f32 = 18.0;
const EMOJI_INLINE_UNDERLINE_OFFSET_Y: f32 = 1.5;
const EMOJI_INLINE_NEGATIVE_SPACE: f32 = -3.0;

const HEADING_LEVEL_1: u8 = 1;
const HEADING_LEVEL_2: u8 = 2;
const HEADING_LEVEL_3: u8 = 3;

impl<'a> HtmlRenderer<'a> {
    pub fn new(ui: &'a mut egui::Ui, base_dir: &'a Path) -> Self {
        let max_w = ui.available_width();
        Self {
            ui,
            _base_dir: base_dir,
            text_color: None,
            max_image_width: max_w,
            is_strong: false,
            is_italics: false,
        }
    }

    pub fn text_color(mut self, color: egui::Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn max_image_width(mut self, width: f32) -> Self {
        self.max_image_width = width;
        self
    }

    pub fn render(mut self, nodes: &[HtmlNode]) -> Option<LinkAction> {
        self.render_nodes(nodes)
    }

    fn render_nodes(&mut self, nodes: &[HtmlNode]) -> Option<LinkAction> {
        let mut action: Option<LinkAction> = None;
        let mut inline_batch: Vec<&HtmlNode> = Vec::new();

        for (i, node) in nodes.iter().enumerate() {
            if node.is_block() {
                if let Some(a) = self.flush_inline_batch(&inline_batch) {
                    action = Some(a);
                }
                inline_batch.clear();

                if let Some(a) = self.render_block(node) {
                    action = Some(a);
                }

                if i < nodes.len() - 1 {
                    self.ui.add_space(block_margin_for(node));
                }
            } else {
                inline_batch.push(node);
            }
        }

        if let Some(a) = self.flush_inline_batch(&inline_batch) {
            action = Some(a);
        }

        action
    }

    pub(super) fn new_inner(
        ui: &'a mut egui::Ui,
        text_color: Option<egui::Color32>,
        max_w: f32,
    ) -> Self {
        Self {
            ui,
            _base_dir: Path::new(""),
            text_color,
            max_image_width: max_w,
            is_strong: false,
            is_italics: false,
        }
    }
}

pub(super) fn collect_text(nodes: &[HtmlNode]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            HtmlNode::Text(t) => s.push_str(t),
            HtmlNode::Link { children, .. }
            | HtmlNode::Heading { children, .. }
            | HtmlNode::Paragraph { children, .. }
            | HtmlNode::Emphasis(children)
            | HtmlNode::Strong(children) => s.push_str(&collect_text(children)),
            HtmlNode::Image { alt, .. } => s.push_str(alt),
            HtmlNode::LineBreak => s.push('\n'),
        }
    }
    s
}

const UTF8_MAX_LEN: usize = 4;
fn encode_unicode_uri(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for c in url.chars() {
        if c.is_ascii() {
            if c == ' ' {
                out.push_str("%20");
            } else {
                out.push(c);
            }
        } else {
            let mut buf = [0; UTF8_MAX_LEN];
            for &b in c.encode_utf8(&mut buf).as_bytes() {
                use std::fmt::Write;
                let _ = write!(&mut out, "%{:02X}", b);
            }
        }
    }
    out
}

fn ensure_svg_extension(url: &str) -> String {
    let encoded = encode_unicode_uri(url);
    let (path, suffix) = split_url_suffix(&encoded);
    if path.ends_with(".svg") {
        return encoded;
    }
    for host in svg_badge_hosts() {
        if encoded.contains(host) {
            return format!("{path}.svg{suffix}");
        }
    }
    encoded
}

fn split_url_suffix(url: &str) -> (&str, &str) {
    let suffix_start = url
        .find('?')
        .into_iter()
        .chain(url.find('#'))
        .min()
        .unwrap_or(url.len());
    url.split_at(suffix_start)
}

fn block_margin_for(node: &katana_core::html::HtmlNode) -> f32 {
    match node {
        katana_core::html::HtmlNode::Heading { .. } => HEADING_BLOCK_MARGIN_Y,
        katana_core::html::HtmlNode::Paragraph { .. } => PARAGRAPH_BLOCK_MARGIN_Y,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests;
