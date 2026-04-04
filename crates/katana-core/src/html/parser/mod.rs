pub mod inline;
pub mod regex;
#[cfg(test)]
mod tests;
pub mod types;

use crate::html::node::{HtmlNode, LinkTarget, TextAlign};
use ::regex::{Captures, Match};
use std::path::Path;
pub use types::*;

impl<'a> HtmlParser<'a> {
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }

    pub fn parse(&self, html: &str) -> Vec<HtmlNode> {
        self.parse_fragment(html)
    }

    fn parse_fragment(&self, html: &str) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        let mut remaining = html;

        while !remaining.is_empty() {
            if let Some(tag_start) = remaining.find('<') {
                self.parse_inline_before_tag(&mut nodes, remaining, tag_start);
                remaining = self.process_tag_or_skip(&mut nodes, &remaining[tag_start..]);
            } else {
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    HtmlInlineParserOps::parse(trimmed, self.base_dir, &mut nodes);
                }
                break;
            }
        }
        nodes
    }

    fn parse_inline_before_tag(
        &self,
        nodes: &mut Vec<HtmlNode>,
        remaining: &str,
        tag_start: usize,
    ) {
        let trimmed = remaining[..tag_start].trim();
        if !trimmed.is_empty() {
            HtmlInlineParserOps::parse(trimmed, self.base_dir, nodes);
        }
    }

    fn process_tag_or_skip<'b>(&self, nodes: &mut Vec<HtmlNode>, remaining: &'b str) -> &'b str {
        if let Some((node, consumed)) = self.try_parse_tag(remaining) {
            nodes.push(node);
            &remaining[consumed..]
        } else if let Some(end) = remaining.find('>') {
            &remaining[end + 1..]
        } else {
            ""
        }
    }

    fn try_parse_tag(&self, s: &str) -> Option<(HtmlNode, usize)> {
        self.try_parse_br_or_img(s)
            .or_else(|| self.try_parse_a(s))
            .or_else(|| self.try_parse_paragraph(s))
            .or_else(|| self.try_parse_heading(s))
            .or_else(|| self.try_parse_em_or_strong(s))
    }

    fn try_parse_br_or_img(&self, s: &str) -> Option<(HtmlNode, usize)> {
        if let Some(m) = HtmlRegexOps::br().find(s)
            && m.start() == 0
        {
            return Some((HtmlNode::LineBreak, m.end()));
        }
        if let Some(caps) = HtmlRegexOps::img().captures(s)
            && caps.get(0).map(|m: Match| m.start()).unwrap_or(1) == 0
        {
            let attrs = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
            let src = HtmlRegexOps::extract_attr(attrs, "src").unwrap_or_default();
            let alt = HtmlRegexOps::extract_attr(attrs, "alt").unwrap_or_default();
            return Some((HtmlNode::Image { src, alt }, caps.get(0).unwrap().end()));
        }
        None
    }

    fn try_parse_a(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::a().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }
        let href = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        let inner = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let children = self.parse_fragment(inner);
        let target = LinkTarget::resolve(href, self.base_dir);
        Some((
            HtmlNode::Link { target, children },
            caps.get(0).unwrap().end(),
        ))
    }

    fn try_parse_paragraph(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::p().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let attrs = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        let inner = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let align =
            HtmlRegexOps::extract_attr(attrs, "align").and_then(|a: String| match a.as_str() {
                "center" => Some(TextAlign::Center),
                "left" => Some(TextAlign::Left),
                "right" => Some(TextAlign::Right),
                _ => None,
            });
        let children = self.parse_fragment(inner);
        Some((
            HtmlNode::Paragraph { align, children },
            caps.get(0).unwrap().end(),
        ))
    }

    fn try_parse_heading(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::heading().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let level: u8 = caps
            .get(1)
            .map(|m: Match| m.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let attrs = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let align = if attrs.contains(r#"align="center""#) {
            Some(TextAlign::Center)
        } else if attrs.contains(r#"align="left""#) {
            Some(TextAlign::Left)
        } else if attrs.contains(r#"align="right""#) {
            Some(TextAlign::Right)
        } else {
            None
        };
        const CAP_HEADING_INNER: usize = 3;
        let inner = caps
            .get(CAP_HEADING_INNER)
            .map(|m: Match| m.as_str())
            .unwrap_or("");
        let children = self.parse_fragment(inner);
        Some((
            HtmlNode::Heading {
                level,
                align,
                children,
            },
            caps.get(0).unwrap().end(),
        ))
    }

    fn try_parse_em_or_strong(&self, s: &str) -> Option<(HtmlNode, usize)> {
        if let Some(caps) = HtmlRegexOps::em().captures(s)
            && caps.get(0).map(|m: Match| m.start()).unwrap_or(1) == 0
        {
            let inner = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
            let children = self.parse_fragment(inner);
            return Some((HtmlNode::Emphasis(children), caps.get(0).unwrap().end()));
        }
        if let Some(caps) = HtmlRegexOps::strong().captures(s)
            && caps.get(0).map(|m: Match| m.start()).unwrap_or(1) == 0
        {
            let inner = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
            let children = self.parse_fragment(inner);
            return Some((HtmlNode::Strong(children), caps.get(0).unwrap().end()));
        }
        None
    }
}
