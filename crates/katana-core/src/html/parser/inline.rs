use crate::html::node::{HtmlNode, LinkTarget};
use std::path::Path;

/// Parses plain text that may contain markdown inline syntax.
///
/// Handles:
/// - `![alt](src)` → Image
/// - `[text](url)` → Link
/// - Plain text → Text
pub fn parse_inline_text(text: &str, base_dir: &Path, nodes: &mut Vec<HtmlNode>) {
    let mut remaining = text;

    while !remaining.is_empty() {
        // WHY: Look for markdown image or link
        let img_pos = remaining.find("![");
        let link_pos = remaining.find('[').filter(|&pos| {
            // WHY: Exclude the '[' that's part of '!['
            img_pos != Some(pos.saturating_sub(1))
        });

        let next_syntax = match (img_pos, link_pos) {
            (Some(i), Some(l)) => Some(i.min(l)),
            (Some(i), None) => Some(i),
            (None, Some(l)) => Some(l),
            (None, None) => None,
        };

        if let Some(pos) = next_syntax {
            // WHY: Text before the syntax
            if pos > 0 {
                nodes.push(HtmlNode::Text(remaining[..pos].to_string()));
            }

            remaining = &remaining[pos..];

            // WHY: Try markdown image: ![alt](src)
            if remaining.starts_with("![") {
                if let Some((node, consumed)) = try_parse_md_image(remaining) {
                    nodes.push(node);
                    remaining = &remaining[consumed..];
                    continue;
                }
            }

            // WHY: Try markdown link: [text](url)
            if remaining.starts_with('[') {
                if let Some((node, consumed)) = try_parse_md_link(remaining, base_dir) {
                    nodes.push(node);
                    remaining = &remaining[consumed..];
                    continue;
                }
            }

            // WHY: Not a valid syntax — emit the character and continue
            nodes.push(HtmlNode::Text(remaining[..1].to_string()));
            remaining = &remaining[1..];
        } else {
            // WHY: No more markdown syntax
            nodes.push(HtmlNode::Text(remaining.to_string()));
            break;
        }
    }
}

/// Tries to parse `![alt](src)` at the start of `s`.
pub fn try_parse_md_image(s: &str) -> Option<(HtmlNode, usize)> {
    let rest = s.strip_prefix("![")?;
    let close_bracket = rest.find("](")?;
    let alt = &rest[..close_bracket];
    let after = &rest[close_bracket + 2..];
    let close_paren = after.find(')')?;
    let src = &after[..close_paren];
    if src.is_empty() {
        return None;
    }
    let total = 2 + close_bracket + 2 + close_paren + 1;
    Some((
        HtmlNode::Image {
            src: src.to_string(),
            alt: alt.to_string(),
        },
        total,
    ))
}

/// Tries to parse `[text](url)` at the start of `s`.
pub fn try_parse_md_link(s: &str, base_dir: &Path) -> Option<(HtmlNode, usize)> {
    let rest = s.strip_prefix('[')?;
    let close_bracket = rest.find("](")?;
    let link_text = &rest[..close_bracket];
    let after = &rest[close_bracket + 2..];
    let close_paren = after.find(')')?;
    let url = &after[..close_paren];
    let total = 1 + close_bracket + 2 + close_paren + 1;
    let target = LinkTarget::resolve(url, base_dir);
    Some((
        HtmlNode::Link {
            target,
            children: vec![HtmlNode::Text(link_text.to_string())],
        },
        total,
    ))
}
