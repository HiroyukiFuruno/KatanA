//! HTML parser that converts HTML fragments into `HtmlNode` trees.
//!
//! Works with `comrak` AST's `HtmlBlock` / `HtmlInline` content,
//! extracting tag attributes via regex for shallow-nested HTML structures.

pub mod inline;
pub mod regex;
#[cfg(test)]
mod tests;

use std::path::Path;

use crate::html::node::{HtmlNode, LinkTarget, TextAlign};

/// Parser that converts HTML strings into structured `HtmlNode` trees.
///
/// Holds the base directory for resolving relative link paths.
pub struct HtmlParser<'a> {
    base_dir: &'a Path,
}

impl<'a> HtmlParser<'a> {
    /// Creates a new parser with the given base directory for link resolution.
    pub fn new(base_dir: &'a Path) -> Self {
        Self { base_dir }
    }

    /// Parses an HTML fragment into a list of `HtmlNode`s.
    ///
    /// The input is typically the content of a `<p align="center">...</p>` block
    /// or similar HTML extracted from a Markdown document.
    pub fn parse(&self, html: &str) -> Vec<HtmlNode> {
        self.parse_fragment(html)
    }

    /// Recursively parses an HTML fragment into nodes.
    fn parse_fragment(&self, html: &str) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        let mut remaining = html;

        while !remaining.is_empty() {
            if let Some(tag_start) = remaining.find('<') {
                // WHY: Text before the tag
                let before = &remaining[..tag_start];
                let trimmed = before.trim();
                if !trimmed.is_empty() {
                    inline::parse_inline_text(trimmed, self.base_dir, &mut nodes);
                }

                remaining = &remaining[tag_start..];

                // WHY: Try to parse a known tag
                if let Some((node, consumed)) = self.try_parse_tag(remaining) {
                    nodes.push(node);
                    remaining = &remaining[consumed..];
                } else {
                    // WHY: Skip unknown/malformed tag — find the end '>'
                    if let Some(end) = remaining.find('>') {
                        remaining = &remaining[end + 1..];
                    } else {
                        break;
                    }
                }
            } else {
                // WHY: No more tags — parse remaining as inline text
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    inline::parse_inline_text(trimmed, self.base_dir, &mut nodes);
                }
                break;
            }
        }

        nodes
    }

    /// Tries to parse a known HTML tag at the beginning of `s`.
    /// Returns `(HtmlNode, bytes_consumed)`.
    fn try_parse_tag(&self, s: &str) -> Option<(HtmlNode, usize)> {
        // WHY: <br> / <br/>
        if let Some(m) = regex::regex_br().find(s) {
            if m.start() == 0 {
                return Some((HtmlNode::LineBreak, m.end()));
            }
        }

        // WHY: <img ...>
        if let Some(caps) = regex::regex_img().captures(s) {
            if caps.get(0)?.start() == 0 {
                let attrs = caps.get(1)?.as_str();
                let src = regex::extract_attr(attrs, "src").unwrap_or_default();
                let alt = regex::extract_attr(attrs, "alt").unwrap_or_default();
                return Some((HtmlNode::Image { src, alt }, caps.get(0)?.end()));
            }
        }

        // WHY: <a href="...">...</a>
        if let Some(caps) = regex::regex_a().captures(s) {
            if caps.get(0)?.start() == 0 {
                let href = caps.get(1)?.as_str();
                let inner = caps.get(2)?.as_str();
                let children = self.parse_fragment(inner);
                let target = LinkTarget::resolve(href, self.base_dir);
                return Some((HtmlNode::Link { target, children }, caps.get(0)?.end()));
            }
        }

        // WHY: <p align="...">...</p>
        if let Some(caps) = regex::regex_p().captures(s) {
            if caps.get(0)?.start() == 0 {
                let attrs = caps.get(1)?.as_str();
                let inner = caps.get(2)?.as_str();
                let align = regex::extract_attr(attrs, "align").and_then(|a| match a.as_str() {
                    "center" => Some(TextAlign::Center),
                    "left" => Some(TextAlign::Left),
                    "right" => Some(TextAlign::Right),
                    _ => None,
                });
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Paragraph { align, children }, caps.get(0)?.end()));
            }
        }

        // WHY: <h1>...<h6>
        if let Some(caps) = regex::regex_heading().captures(s) {
            if caps.get(0)?.start() == 0 {
                let level: u8 = caps.get(1)?.as_str().parse().ok()?;
                let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
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
                let inner = caps.get(CAP_HEADING_INNER)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((
                    HtmlNode::Heading {
                        level,
                        align,
                        children,
                    },
                    caps.get(0)?.end(),
                ));
            }
        }

        // WHY: <em>...</em>
        if let Some(caps) = regex::regex_em().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Emphasis(children), caps.get(0)?.end()));
            }
        }

        // WHY: <strong>...</strong>
        if let Some(caps) = regex::regex_strong().captures(s) {
            if caps.get(0)?.start() == 0 {
                let inner = caps.get(1)?.as_str();
                let children = self.parse_fragment(inner);
                return Some((HtmlNode::Strong(children), caps.get(0)?.end()));
            }
        }

        None
    }
}
