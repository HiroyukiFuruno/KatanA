/* WHY: Verification of nested elements, such as links containing images (common in README badges) and aligned paragraphs. */

use super::*;
use crate::html::parser::TextAlign;
use crate::html::{HtmlNode, LinkTarget};

#[test]
fn parse_shields_badge_pattern() {
    let html = r#"<a href="https://github.com/org/repo/actions"><img src="https://img.shields.io/github/actions/workflow/status/org/repo/ci.yml?label=CI" alt="CI"></a>"#;
    let nodes = parser().parse(html);
    assert_eq!(nodes.len(), 1);
    let HtmlNode::Link { target, children } = &nodes[0] else {
        panic!("Expected Link node");
    };
    assert_eq!(
        *target,
        LinkTarget::External("https://github.com/org/repo/actions".into())
    );
    assert_eq!(children.len(), 1);
    let HtmlNode::Image { src, alt } = &children[0] else {
        panic!("Expected Image inside Link");
    };
    assert!(src.contains("img.shields.io"));
    assert_eq!(alt, "CI");
}

#[test]
fn parse_centered_paragraph_with_badges() {
    let html = r#"<p align="center">
  <a href="LICENSE"><img src="badge1.svg" alt="License"></a>
  <a href="actions"><img src="badge2.svg" alt="CI"></a>
</p>"#;
    let nodes = parser().parse(html);
    assert_eq!(nodes.len(), 1);
    let HtmlNode::Paragraph { align, children } = &nodes[0] else {
        panic!("Expected Paragraph");
    };
    assert_eq!(*align, Some(TextAlign::Center));
    let links: Vec<_> = children
        .iter()
        .filter(|n| matches!(n, HtmlNode::Link { .. }))
        .collect();
    assert_eq!(links.len(), 2);
}

#[test]
fn parse_readme_badge_block_structure() {
    let html = r#"<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/KatanA/actions/workflows/ci.yml"><img src="https://github.com/HiroyukiFuruno/KatanA/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/HiroyukiFuruno/KatanA/releases/latest"><img src="https://img.shields.io/github/v/release/HiroyukiFuruno/KatanA" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS">
</p>"#;
    let nodes = parser().parse(html);
    assert_eq!(nodes.len(), 1, "Should produce 1 Paragraph node");

    let HtmlNode::Paragraph { align, children } = &nodes[0] else {
        panic!("Expected Paragraph");
    };
    assert_eq!(*align, Some(TextAlign::Center));
    assert_eq!(
        children.len(),
        4,
        "Should have 4 children (3 links + 1 image)"
    );

    /* WHY: All children must be inline */
    for (i, c) in children.iter().enumerate() {
        assert!(c.is_inline(), "Child {i} should be inline, got {:?}", c);
    }
}
