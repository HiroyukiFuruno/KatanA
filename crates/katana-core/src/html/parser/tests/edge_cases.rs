/* WHY: Verification of robustness against malformed HTML, unknown tags, and ambiguous Markdown inline syntax. */

use super::*;
use crate::html::HtmlNode;
use crate::html::parser::TextAlign;

#[test]
fn parse_unknown_tag_is_skipped() {
    let nodes = parser().parse("<div>content</div>");
    /* WHY: Unknown tag is skipped; content text is parsed */
    assert!(!nodes.is_empty());
}

#[test]
fn parse_malformed_tag_without_closing_bracket() {
    /* WHY: Malformed tag without '>' should not loop forever */
    let nodes = parser().parse("<unclosed");
    assert!(nodes.is_empty());
}

#[test]
fn parse_text_before_known_tag() {
    let nodes = parser().parse("hello <br>");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0], HtmlNode::Text("hello".into()));
    assert_eq!(nodes[1], HtmlNode::LineBreak);
}

#[test]
fn parse_paragraph_left_align() {
    let html = r#"<p align="left">text</p>"#;
    let nodes = parser().parse(html);
    let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
        panic!("Expected Paragraph");
    };
    assert_eq!(*align, Some(TextAlign::Left));
}

#[test]
fn parse_paragraph_right_align() {
    let html = r#"<p align="right">text</p>"#;
    let nodes = parser().parse(html);
    let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
        panic!("Expected Paragraph");
    };
    assert_eq!(*align, Some(TextAlign::Right));
}

#[test]
fn parse_paragraph_unknown_align_is_none() {
    let html = r#"<p align="justify">text</p>"#;
    let nodes = parser().parse(html);
    let HtmlNode::Paragraph { align, .. } = &nodes[0] else {
        panic!("Expected Paragraph");
    };
    assert_eq!(*align, None);
}

#[test]
fn parse_md_image_with_empty_src_returns_text() {
    /* WHY: ![alt]() has empty src → should NOT create Image node */
    let nodes = parser().parse("![alt]()");
    /* WHY: Falls through to text output */
    assert!(nodes.iter().all(|n| !matches!(n, HtmlNode::Image { .. })));
}

#[test]
fn parse_mixed_md_image_and_link() {
    /* WHY: Link before image → both `[` and `![` are found → covers min(img_pos, link_pos) */
    let nodes = parser().parse("[click](https://example.com) ![icon](a.png)");
    let images: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n, HtmlNode::Image { .. }))
        .collect();
    let links: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n, HtmlNode::Link { .. }))
        .collect();
    assert_eq!(images.len(), 1, "Should have one image");
    assert_eq!(links.len(), 1, "Should have one link");
}

#[test]
fn parse_bracket_not_followed_by_paren_emits_text() {
    /* WHY: Lone '[' that isn't a valid link → should emit as text character */
    let nodes = parser().parse("[not a link");
    assert!(!nodes.is_empty());
    /* WHY: Should contain text, not a link */
    assert!(nodes.iter().all(|n| !matches!(n, HtmlNode::Link { .. })));
}
