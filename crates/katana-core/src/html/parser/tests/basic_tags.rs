/* WHY: Verification of standard HTML tag parsing (img, br, p, headings, formatting). */

use super::*;
use crate::html::parser::TextAlign;
use crate::html::{HtmlNode, LinkTarget};
use std::path::PathBuf;

#[test]
fn parse_img_tag() {
    let nodes = parser().parse(r#"<img src="icon.png" alt="icon">"#);
    assert_eq!(
        nodes,
        vec![HtmlNode::Image {
            src: "icon.png".into(),
            alt: "icon".into(),
        }]
    );
}

#[test]
fn parse_br_tag() {
    let nodes = parser().parse("<br>");
    assert_eq!(nodes, vec![HtmlNode::LineBreak]);
}

#[test]
fn parse_br_self_closing() {
    let nodes = parser().parse("<br/>");
    assert_eq!(nodes, vec![HtmlNode::LineBreak]);
}

#[test]
fn parse_link_with_image_badge() {
    let html = r#"<a href="LICENSE"><img src="badge.svg" alt="License"></a>"#;
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Link {
            target: LinkTarget::InternalFile(PathBuf::from("/project/LICENSE")),
            children: vec![HtmlNode::Image {
                src: "badge.svg".into(),
                alt: "License".into(),
            }],
        }]
    );
}

#[test]
fn parse_centered_paragraph() {
    let html = r#"<p align="center">hello</p>"#;
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Paragraph {
            align: Some(TextAlign::Center),
            children: vec![HtmlNode::Text("hello".into())],
        }]
    );
}

#[test]
fn parse_heading() {
    let html = "<h2>Section Title</h2>";
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Heading {
            level: 2,
            align: None,
            children: vec![HtmlNode::Text("Section Title".into())],
        }]
    );
}

#[test]
fn parse_centered_heading() {
    let html = r#"<h1 align="center">Title</h1>"#;
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Heading {
            level: 1,
            align: Some(TextAlign::Center),
            children: vec![HtmlNode::Text("Title".into())],
        }]
    );
}

#[test]
fn parse_emphasis() {
    let html = "<em>italic</em>";
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Emphasis(vec![HtmlNode::Text("italic".into()),])]
    );
}

#[test]
fn parse_strong() {
    let html = "<strong>bold</strong>";
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Strong(vec![HtmlNode::Text("bold".into()),])]
    );
}

#[test]
fn parse_left_heading() {
    let html = r#"<h3 align="left">Left</h3>"#;
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Heading {
            level: 3,
            align: Some(TextAlign::Left),
            children: vec![HtmlNode::Text("Left".into())],
        }]
    );
}

#[test]
fn parse_right_heading() {
    let html = r#"<h6 align="right">Right</h6>"#;
    let nodes = parser().parse(html);
    assert_eq!(
        nodes,
        vec![HtmlNode::Heading {
            level: 6,
            align: Some(TextAlign::Right),
            children: vec![HtmlNode::Text("Right".into())],
        }]
    );
}
